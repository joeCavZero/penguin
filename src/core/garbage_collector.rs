use crate::core::cell::*;
use crate::core::colour::*;
use crate::core::env::*;
use crate::core::function::*;
use crate::core::heap_value::*;
use crate::core::thread::*;
use crate::core::utils::*;
use crate::core::value::*;

pub trait PengGarbageCollectable {
    fn mark_and_sweep(&mut self);
}

impl PengGarbageCollectable for PengEnv {
    fn mark_and_sweep(&mut self) {
        paint_all_red(self);

        let roots = collect_roots(self);

        for root in roots {
            mark_ptr(self, root);
        }

        sweep(self);
    }
}

fn paint_all_red(env: &mut PengEnv) {
    for (_, heap_value) in env.heap.iter_mut() {
        heap_value.colour = PengColour::Red;
    }
}

fn collect_roots(env: &PengEnv) -> Vec<PengHeapPtr> {
    let mut roots = Vec::new();

    for (ptr, heap_value) in env.heap.iter() {
        if let PengHeapValue::Thread(thread) = &heap_value.value {
            match thread.state {
                PengThreadState::Running | PengThreadState::Paused | PengThreadState::Waiting => {
                    roots.push(*ptr);
                }

                PengThreadState::Finished
                | PengThreadState::Failed
                | PengThreadState::Cancelled => {}
            }
        }
    }

    roots
}

fn mark_ptr(env: &mut PengEnv, ptr: PengHeapPtr) {
    let children = {
        let heap_value = match env.get_coloured_heap_mut(ptr) {
            Some(value) => value,
            None => return,
        };

        if heap_value.colour == PengColour::Black {
            return;
        }

        heap_value.colour = PengColour::Black;

        collect_children(&heap_value.value)
    };

    for child in children {
        mark_ptr(env, child);
    }
}

fn collect_children(value: &PengHeapValue) -> Vec<PengHeapPtr> {
    let mut children = Vec::new();

    collect_heap_value_children(value, &mut children);

    children
}

fn sweep(env: &mut PengEnv) {
    env.heap
        .retain(|_, heap_value| heap_value.colour == PengColour::Black);
}

fn collect_value_children(value: &PengValue, children: &mut Vec<PengHeapPtr>) {
    match value {
        PengValue::Cell(cell) => {
            collect_cell_children(cell, children);
        }

        PengValue::Heap(heap_value) => {
            collect_heap_value_children(heap_value, children);
        }
    }
}

fn collect_cell_children(cell: &PengCell, children: &mut Vec<PengHeapPtr>) {
    if let PengCell::Reference(ptr) = cell {
        children.push(*ptr);
    }
}

fn collect_heap_value_children(value: &PengHeapValue, children: &mut Vec<PengHeapPtr>) {
    match value {
        PengHeapValue::String(_) => {}

        PengHeapValue::Vector(vector) => {
            for item in vector.values.iter() {
                collect_cell_children(item.value(), children);
            }
        }

        PengHeapValue::Object(object) => {
            for (_, item) in object.fields.iter() {
                collect_cell_children(item.value(), children);
            }
        }

        PengHeapValue::Thread(thread) => {
            for cell in thread.stack.iter() {
                collect_cell_children(cell.value(), children);
            }

            for frame in thread.frames.iter() {
                children.push(frame.procedure_ptr);
            }

            match &thread.result {
                PengThreadResult::Returned(cell) => {
                    collect_cell_children(cell.value(), children);
                }

                PengThreadResult::Pending | PengThreadResult::Failed(_) => {}
            }
        }

        PengHeapValue::Function(function) => match function {
            PengFunction::Bytecode(func) => {
                for ptr in func.using_values.iter() {
                    children.push(*ptr);
                }

                for constant in func.consts.iter() {
                    collect_value_children(constant, children);
                }

                for instruction in func.bytecode.iter() {
                    collect_instruction_children(instruction, children);
                }
            }

            PengFunction::Native(_) => {}
        },

        PengHeapValue::Module(module) => {
            for (_, member) in module.members.iter() {
                collect_cell_children(member.value(), children);
            }
        }

        PengHeapValue::Type(_) => {}

        PengHeapValue::Operation(_) => {}

        PengHeapValue::Union(_) => {}
    }
}

use crate::core::instruction::*;

fn collect_instruction_children(instruction: &PengInstruction, children: &mut Vec<PengHeapPtr>) {
    match instruction {
        PengInstruction::PushHeap(ptr) => {
            children.push(*ptr);
        }

        _ => {}
    }
}

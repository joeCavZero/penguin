use crate::core::cell::*;
use crate::core::colour::*;
use crate::core::env::*;
use crate::core::function::*;
use crate::core::heap_value::*;
use crate::core::instruction::*;
use crate::core::operation::*;
use crate::core::thread::*;
use crate::core::typing::*;
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
    for (_, heap_value) in env.heap_mut().iter_mut() {
        heap_value.colour = PengColour::Red;
    }
}

fn collect_roots(env: &PengEnv) -> Vec<PengHeapPtr> {
    let mut roots = Vec::new();

    for ptr in env.pinned().iter() {
        roots.push(*ptr);
    }

    for ptr in env.active_threads().iter() {
        roots.push(*ptr);
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

fn collect_children(value: &PengValue) -> Vec<PengHeapPtr> {
    let mut children = Vec::new();

    collect_value_children(value, &mut children);

    children
}

fn sweep(env: &mut PengEnv) {
    env.heap_mut()
        .retain(|_, heap_value| heap_value.colour == PengColour::Black);
}

fn collect_value_children(value: &PengValue, children: &mut Vec<PengHeapPtr>) {
    match value {
        PengValue::Cell(cell) => {
            collect_cell_children(cell, children);
        }

        PengValue::Box(value) => {
            collect_box_children(value, children);
        }
    }
}

fn collect_cell_children(cell: &PengCell, children: &mut Vec<PengHeapPtr>) {
    if let PengCell::Reference(ptr) = cell {
        children.push(*ptr);
    }
}

fn collect_box_children(value: &PengBox, children: &mut Vec<PengHeapPtr>) {
    match value {
        PengBox::String(_) => {}

        PengBox::Vector(vector) => {
            for item in vector.values.iter() {
                collect_cell_children(item.value(), children);
            }
        }

        PengBox::Object(object) => {
            for (_, item) in object.fields.iter() {
                collect_cell_children(item.value(), children);
            }
        }

        PengBox::Thread(thread) => {
            for cell in thread.stack.iter() {
                collect_cell_children(cell.value(), children);
            }

            for frame in thread.frames.iter() {
                children.push(frame.procedure);
            }

            match &thread.result {
                PengThreadResult::Returned(cell) => {
                    collect_cell_children(cell.value(), children);
                }

                PengThreadResult::Pending | PengThreadResult::Failed(_) => {}
            }
        }

        PengBox::Function(function) => {
            collect_function_children(function, children);
        }

        PengBox::Operation(operation) => {
            collect_operation_children(operation, children);
        }

        PengBox::Module(module) => {
            for (_, member) in module.members.iter() {
                collect_cell_children(member.value(), children);
            }
        }

        PengBox::Type(value) => {
            collect_type_children(value, children);
        }

        PengBox::Union(union) => {
            for value in union.unions.iter() {
                collect_type_children(value, children);
            }
        }
    }
}

fn collect_function_children(function: &PengFunction, children: &mut Vec<PengHeapPtr>) {
    match function {
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
    }
}

fn collect_operation_children(operation: &PengOperation, children: &mut Vec<PengHeapPtr>) {
    match operation {
        PengOperation::Bytecode(operation) => {
            for ptr in operation.using_values.iter() {
                children.push(*ptr);
            }

            for constant in operation.consts.iter() {
                collect_value_children(constant, children);
            }

            for instruction in operation.bytecode.iter() {
                collect_instruction_children(instruction, children);
            }
        }

        PengOperation::Native(_) => {}
    }
}

fn collect_type_children(value: &PengType, children: &mut Vec<PengHeapPtr>) {
    match value {
        PengType::Custom(custom_type) => {
            for (_, item) in custom_type.fields.iter() {
                collect_cell_children(item.value(), children);
            }
        }

        PengType::Vector(inner) => {
            collect_type_children(inner, children);
        }

        _ => {}
    }
}

fn collect_instruction_children(instruction: &PengInstruction, children: &mut Vec<PengHeapPtr>) {
    match instruction {
        PengInstruction::PushHeap(ptr) | PengInstruction::PushHeapRef(ptr) => {
            children.push(*ptr);
        }

        _ => {}
    }
}

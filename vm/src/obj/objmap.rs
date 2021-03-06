use crate::function::Args;
use crate::pyobject::{PyContext, PyObjectRef, PyRef, PyResult, PyValue};
use crate::vm::VirtualMachine;

use super::objiter;
use super::objtype::PyClassRef;

#[derive(Debug)]
pub struct PyMap {
    mapper: PyObjectRef,
    iterators: Vec<PyObjectRef>,
}
type PyMapRef = PyRef<PyMap>;

impl PyValue for PyMap {
    fn class(vm: &VirtualMachine) -> PyClassRef {
        vm.ctx.map_type()
    }
}

fn map_new(
    cls: PyClassRef,
    function: PyObjectRef,
    iterables: Args,
    vm: &VirtualMachine,
) -> PyResult<PyMapRef> {
    let iterators = iterables
        .into_iter()
        .map(|iterable| objiter::get_iter(vm, &iterable))
        .collect::<Result<Vec<_>, _>>()?;
    PyMap {
        mapper: function.clone(),
        iterators,
    }
    .into_ref_with_type(vm, cls.clone())
}

impl PyMapRef {
    fn next(self, vm: &VirtualMachine) -> PyResult {
        let next_objs = self
            .iterators
            .iter()
            .map(|iterator| objiter::call_next(vm, iterator))
            .collect::<Result<Vec<_>, _>>()?;

        // the mapper itself can raise StopIteration which does stop the map iteration
        vm.invoke(self.mapper.clone(), next_objs)
    }

    fn iter(self, _vm: &VirtualMachine) -> Self {
        self
    }
}

pub fn init(context: &PyContext) {
    let map_type = &context.map_type;

    let map_doc = "map(func, *iterables) --> map object\n\n\
                   Make an iterator that computes the function using arguments from\n\
                   each of the iterables.  Stops when the shortest iterable is exhausted.";

    extend_class!(context, map_type, {
        "__new__" => context.new_rustfunc(map_new),
        "__next__" => context.new_rustfunc(PyMapRef::next),
        "__iter__" => context.new_rustfunc(PyMapRef::iter),
        "__doc__" => context.new_str(map_doc.to_string())
    });
}

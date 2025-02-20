extern crate glium;

use glium::buffer::{BufferMutSlice, BufferSlice};
use glium::vertex::{Vertex, VertexBuffer, VertexBufferAny, VerticesSource};
use std::any::TypeId;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::ops::Deref;

/// A type-erased `VertexBuffer` which can be safely borrowed as the original `VertexBuffer<T>`
/// using a dynamic type check.
pub struct TypedVertexBufferAny {
    type_id: TypeId,
    buffer: VertexBufferAny,
}

impl TypedVertexBufferAny {
    /// Borrows buffer immutably as typed, checking the type at runtime.
    ///
    /// Returns an error if the type is not the one given on construction.
    pub fn as_typed_slice<T: Vertex + Send + 'static>(
        &self,
    ) -> Result<BufferSlice<[T]>, IncorrectTypeError> {
        if self.type_id == TypeId::of::<T>() {
            Ok(unsafe { self.buffer.deref().as_typed_slice() })
        } else {
            Err(IncorrectTypeError)
        }
    }

    /// Borrows the buffer mutably as typed, checking the type at runtime.
    ///
    /// Returns an error if the type is not the one given on construction.
    pub fn as_typed_slice_mut<T: Vertex + Send + 'static>(
        &mut self,
    ) -> Result<BufferMutSlice<[T]>, IncorrectTypeError> {
        if self.type_id == TypeId::of::<T>() {
            Ok(unsafe { self.buffer.as_typed_slice_mut() })
        } else {
            Err(IncorrectTypeError)
        }
    }
}

impl<T> From<VertexBuffer<T>> for TypedVertexBufferAny
  where T: Vertex + Send + 'static,
{
    fn from(buffer: VertexBuffer<T>) -> TypedVertexBufferAny {
        TypedVertexBufferAny {
            type_id: TypeId::of::<T>(),
            buffer: buffer.into(),
        }
    }
}

impl<'a> Into<VerticesSource<'a>> for &'a TypedVertexBufferAny {
    fn into(self) -> VerticesSource<'a> {
        (&self.buffer).into()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct IncorrectTypeError;

impl Error for IncorrectTypeError {}

impl Display for IncorrectTypeError {
    fn fmt(&self, formatter: &mut Formatter) -> FmtResult {
        write!(formatter, "incorrect buffer type")
    }
}
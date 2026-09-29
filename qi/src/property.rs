//! Properties: typed values whose changes are notified.
//!
//! A [`Property`] holds a value that may be read and written, and notifies its changes to
//! subscribers like a [`Signal`](crate::Signal). In the `qi` type system, a property is also a
//! signal of the same identifier and name.
//!
//! Like signals, properties are either *local* or *proxies* to the property of a remote object,
//! and share the same API.

use crate::{
    object::ObjectClient,
    signal::{Signal, Subscription, ValueStream},
    value::{self, FromValue, IntoValue, Reflect, RuntimeReflect, Value},
    Result,
};
use std::sync::{Arc, RwLock};

/// A typed property, either local or a proxy to the property of a remote object.
///
/// Properties are cheap to clone: clones of a local property share the same value and signal.
///
/// # Local properties
///
/// ```
/// # tokio_test::block_on(async {
/// use futures::StreamExt;
/// let property = qi::Property::new(1);
/// let mut changes = property.subscribe().await.unwrap();
/// property.set(2).await.unwrap();
/// assert_eq!(property.get().await.unwrap(), 2);
/// assert_eq!(changes.next().await, Some(2));
/// # });
/// ```
pub struct Property<T> {
    inner: PropertyInner<T>,
}

enum PropertyInner<T> {
    Local {
        value: Arc<RwLock<T>>,
        signal: Signal<T>,
    },
    Remote {
        object: ObjectClient,
        id: value::object::ActionId,
        signal: Signal<T>,
    },
}

impl<T> Property<T>
where
    T: Clone
        + Send
        + Sync
        + 'static
        + IntoValue<'static>
        + FromValue<'static>
        + Reflect
        + RuntimeReflect,
{
    /// Creates a local property with an initial value.
    pub fn new(value: T) -> Self {
        Self {
            inner: PropertyInner::Local {
                value: Arc::new(RwLock::new(value)),
                signal: Signal::new(),
            },
        }
    }

    pub(crate) fn remote(object: ObjectClient, id: value::object::ActionId) -> Self {
        let signal = Signal::remote(object.clone(), id);
        Self {
            inner: PropertyInner::Remote { object, id, signal },
        }
    }

    /// Returns true if the property is a proxy to a remote property.
    pub fn is_remote(&self) -> bool {
        matches!(self.inner, PropertyInner::Remote { .. })
    }

    /// Gets the current value of the property.
    pub async fn get(&self) -> Result<T> {
        match &self.inner {
            PropertyInner::Local { value, .. } => Ok(read(value).clone()),
            PropertyInner::Remote { object, id, .. } => object.property_action(*id).await,
        }
    }

    /// Sets the value of the property, notifying subscribers of the change.
    pub async fn set(&self, new_value: T) -> Result<()> {
        match &self.inner {
            PropertyInner::Local { value, signal } => {
                *write(value) = new_value.clone();
                signal.emit(new_value);
                Ok(())
            }
            PropertyInner::Remote { object, id, .. } => {
                object
                    .set_property_action(*id, new_value.into_value())
                    .await
            }
        }
    }

    /// Subscribes to the changes of the property.
    pub async fn subscribe(&self) -> Result<Subscription<T>> {
        self.signal().subscribe().await
    }

    /// The signal of changes of the property.
    pub fn signal(&self) -> &Signal<T> {
        match &self.inner {
            PropertyInner::Local { signal, .. } | PropertyInner::Remote { signal, .. } => signal,
        }
    }

    pub(crate) async fn get_erased(&self) -> Result<Value<'static>> {
        Ok(self.get().await?.into_value())
    }

    pub(crate) async fn set_erased(&self, value: Value<'_>) -> Result<()> {
        let value = T::from_value(value.into_owned())
            .map_err(crate::error::ValueConversionError::Arguments)?;
        self.set(value).await
    }

    /// The type of the values of the property, `None` if dynamic.
    pub fn value_type() -> Option<value::Type> {
        <T as Reflect>::ty()
    }

    pub(crate) async fn subscribe_erased(&self) -> Result<ValueStream> {
        self.signal().subscribe_erased().await
    }
}

impl<T> Default for Property<T>
where
    T: Default
        + Clone
        + Send
        + Sync
        + 'static
        + IntoValue<'static>
        + FromValue<'static>
        + Reflect
        + RuntimeReflect,
{
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T> Clone for Property<T> {
    fn clone(&self) -> Self {
        Self {
            inner: match &self.inner {
                PropertyInner::Local { value, signal } => PropertyInner::Local {
                    value: Arc::clone(value),
                    signal: signal.clone(),
                },
                PropertyInner::Remote { object, id, signal } => PropertyInner::Remote {
                    object: object.clone(),
                    id: *id,
                    signal: signal.clone(),
                },
            },
        }
    }
}

impl<T> std::fmt::Debug for Property<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.inner {
            PropertyInner::Local { value, .. } => f
                .debug_struct("Property")
                .field("kind", &"local")
                .field("value", &*read(value))
                .finish(),
            PropertyInner::Remote { object, id, .. } => f
                .debug_struct("Property")
                .field("kind", &"remote")
                .field("object", object)
                .field("id", id)
                .finish(),
        }
    }
}

fn read<T>(lock: &RwLock<T>) -> std::sync::RwLockReadGuard<'_, T> {
    lock.read().unwrap_or_else(|err| {
        lock.clear_poison();
        err.into_inner()
    })
}

fn write<T>(lock: &RwLock<T>) -> std::sync::RwLockWriteGuard<'_, T> {
    lock.write().unwrap_or_else(|err| {
        lock.clear_poison();
        err.into_inner()
    })
}

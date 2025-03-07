//! Utilities for composing Tower Services.

#![deny(rust_2018_idioms, clippy::disallowed_methods, clippy::disallowed_types)]
#![forbid(unsafe_code)]

mod arc_new_service;
mod box_future;
mod box_service;
mod connect;
mod either;
mod fail;
mod fail_on_error;
pub mod failfast;
mod filter;
pub mod gate;
pub mod layer;
mod lazy;
mod loadshed;
mod map_err;
mod map_target;
pub mod monitor;
pub mod new_service;
mod on_service;
pub mod proxy;
pub mod queue;
mod result;
mod switch_ready;
mod thunk;
mod timeout;
mod unwrap_or;
mod watch;

pub use self::{
    arc_new_service::ArcNewService,
    box_future::BoxFuture,
    box_service::{BoxService, BoxServiceLayer},
    connect::{MakeConnection, WithoutConnectionMetadata},
    either::{Either, NewEither},
    fail::Fail,
    fail_on_error::FailOnError,
    failfast::{FailFast, FailFastError},
    filter::{Filter, FilterLayer, Predicate},
    gate::Gate,
    lazy::{Lazy, NewLazy},
    loadshed::{LoadShed, LoadShedError},
    map_err::{MapErr, MapErrBoxed, NewMapErr, WrapErr},
    map_target::{MapTarget, MapTargetLayer, MapTargetService},
    monitor::{Monitor, MonitorError, MonitorNewService, MonitorService, NewMonitor},
    new_service::{NewCloneService, NewFromTargets, NewFromTargetsInner, NewService},
    on_service::{OnService, OnServiceLayer},
    proxy::Proxy,
    queue::{NewQueue, NewQueueWithoutTimeout, Queue, QueueWithoutTimeout},
    result::ResultService,
    switch_ready::{NewSwitchReady, SwitchReady},
    thunk::{NewThunk, Thunk, ThunkClone},
    timeout::{Timeout, TimeoutError},
    unwrap_or::UnwrapOr,
    watch::{NewSpawnWatch, SpawnWatch},
};
pub use tower::{
    service_fn,
    util::{self, future_service, BoxCloneService, FutureService, Oneshot, ServiceExt},
    Service,
};

pub type BoxFutureService<S, E = linkerd_error::Error> = FutureService<
    std::pin::Pin<Box<dyn std::future::Future<Output = Result<S, E>> + Send + 'static>>,
    S,
>;

/// Describes a stack target that can produce `T` typed parameters.
///
/// Stacks (usually layered `NewService` implementations) frequently need to be
/// able to obtain configuration from the stack target, but stack modules are
/// decoupled from any concrete target types. The `Param` trait provides a way to
/// statically guarantee that a given target can provide a configuration
/// parameter.
pub trait Param<T> {
    /// Produces a `T`-typed stack paramter.
    fn param(&self) -> T;
}

/// A strategy for obtaining a `P`-typed parameters from a `T`-typed target.
///
/// This allows stack modules to be decoupled from whether a parameter is known at construction-time
/// or instantiation-time.
pub trait ExtractParam<P, T> {
    fn extract_param(&self, t: &T) -> P;
}

/// A strategy for setting `P`-typed parameters on a `T`-typed target, potentially altering the
/// target type.
pub trait InsertParam<P, T> {
    type Target;

    fn insert_param(&self, param: P, target: T) -> Self::Target;
}

/// Implements `ExtractParam` by cloning the inner `P`-typed parameter.
#[derive(Copy, Clone, Debug)]
pub struct CloneParam<P>(P);

// === Param ===

/// The identity `Param`.
impl<T: ToOwned> Param<T::Owned> for T {
    #[inline]
    fn param(&self) -> T::Owned {
        self.to_owned()
    }
}

// === ExtractParam ===

impl<F, P, T> ExtractParam<P, T> for F
where
    F: Fn(&T) -> P,
{
    fn extract_param(&self, t: &T) -> P {
        (self)(t)
    }
}

impl<P, T: Param<P>> ExtractParam<P, T> for () {
    fn extract_param(&self, t: &T) -> P {
        t.param()
    }
}

// === impl CloneParam ===

impl<P> From<P> for CloneParam<P> {
    fn from(p: P) -> Self {
        Self(p)
    }
}

impl<P: ToOwned, T> ExtractParam<P::Owned, T> for CloneParam<P> {
    #[inline]
    fn extract_param(&self, _: &T) -> P::Owned {
        self.0.to_owned()
    }
}

// === InsertParam ===

impl<P, T> InsertParam<P, T> for () {
    type Target = (P, T);

    #[inline]
    fn insert_param(&self, param: P, target: T) -> (P, T) {
        (param, target)
    }
}

impl<F, P, T, U> InsertParam<P, T> for F
where
    F: Fn(P, T) -> U,
{
    type Target = U;

    #[inline]
    fn insert_param(&self, param: P, target: T) -> U {
        (self)(param, target)
    }
}

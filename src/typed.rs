use crate::{HttpRequest, HttpResponse, HttpResponseError};
use bevy::ecs::bundle::Bundle;
use bevy::ecs::query::{Added, QueryData};
use bevy::prelude::{App, Update};
use bevy::prelude::{Commands, Component, Entity, Query};
use ehttp::{Request, Response};
use serde::Deserialize;
use std::marker::PhantomData;

/// function required to call in order to add TypedResponse once request is finished
pub fn register_request_type<T: Send + Sync + 'static>(app: &mut App) -> &mut App {
    app.add_systems(Update, handle_typed_response::<T>)
}

/// RequestBundle provides easy way to create request that after
/// completing it will add TypedResponse with T type
#[derive(Bundle, Debug, Clone)]
pub struct RequestBundle<T>
where
    T: Send + Sync + 'static,
{
    /// request that will be wrapped up
    pub request: HttpRequest,
    pub request_type: RequestType<T>,
}

impl<T> RequestBundle<T>
where
    T: Send + Sync + 'static,
{
    /// Recomended way to create a new RequestBundle of a given type.
    pub fn new(request: Request) -> Self {
        Self {
            request: HttpRequest(request),
            request_type: RequestType::<T>(PhantomData),
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct RequestType<T>(pub PhantomData<T>);

/// wrap for ehttp response
#[derive(Component, Clone, Debug)]
pub struct TypedResponse<T>
where
    T: Send + Sync,
{
    pub result: Result<Response, String>,
    res: PhantomData<T>,
}

impl<T> TypedResponse<T>
where
    T: for<'a> Deserialize<'a> + Send + Sync,
{
    pub fn parse(&self) -> Option<T> {
        if let Ok(response) = &self.result {
            match response.text() {
                Some(s) => match serde_json::from_str::<T>(s) {
                    Ok(val) => Some(val),
                    _ => None,
                },
                None => None,
            }
        } else {
            None
        }
    }
}

#[derive(QueryData)]
#[query_data(mutable, derive(Debug))]
pub struct TypedRequestQuery<T: Send + Sync + 'static> {
    pub entity: Entity,
    pub response: &'static HttpResponse,
    pub type_info: &'static RequestType<T>,
}

#[derive(QueryData)]
#[query_data(mutable, derive(Debug))]
pub struct TypedFailedRequestQuery<T: Send + Sync + 'static> {
    pub entity: Entity,
    pub response: &'static HttpResponseError,
    pub type_info: &'static RequestType<T>,
}

pub fn handle_typed_response<T: Send + Sync + 'static>(
    mut commands: Commands,
    request_tasks: Query<TypedRequestQuery<T>, Added<HttpResponse>>,
    failed_tasks: Query<TypedFailedRequestQuery<T>, Added<HttpResponseError>>,
) {
    for entry in request_tasks.iter() {
        commands
            .entity(entry.entity)
            .insert(TypedResponse::<T> {
                result: Ok(entry.response.0.clone()),
                res: PhantomData,
            })
            .remove::<RequestType<T>>();
    }
    for entry in failed_tasks.iter() {
        commands
            .entity(entry.entity)
            .insert(TypedResponse::<T> {
                result: Err(entry.response.0.clone()),
                res: PhantomData,
            })
            .remove::<RequestType<T>>();
    }
}
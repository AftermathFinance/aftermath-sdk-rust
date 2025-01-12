use cynic::QueryFragment;
use serde::Deserialize;

use super::fragments::{ObjectFilter, PageInfoForward};
use crate::{schema, Paged};

/// Paged object data where the `Object` fragment does take any parameters.
#[derive(cynic::QueryFragment, Clone, Debug)]
#[cynic(variables = "Variables")]
pub struct Query<Object>
where
    Object: QueryFragment<SchemaType = schema::Object, VariablesFields = ()>,
{
    #[arguments(filter: $filter, first: $first, after: $after)]
    pub objects: ObjectConnection<Object>,
}

impl<Object> Query<Object> where
    Object: QueryFragment<SchemaType = schema::Object, VariablesFields = ()>
{
}

impl<Object> Paged for Query<Object>
where
    Object: for<'de> Deserialize<'de>
        + QueryFragment<SchemaType = schema::Object, VariablesFields = ()>
        + Send,
{
    type Input = Variables;

    type NextInput = Variables;

    type NextPage = Self;

    fn next_variables(&self, mut prev_vars: Self::Input) -> Option<Self::NextInput> {
        let PageInfoForward {
            has_next_page,
            end_cursor,
        } = &self.objects.page_info;
        if *has_next_page {
            prev_vars.after.clone_from(end_cursor);
            Some(prev_vars)
        } else {
            None
        }
    }
}

// =============================================================================
//  Variables
// =============================================================================

#[derive(cynic::QueryVariables, Clone, Debug)]
pub struct Variables {
    pub filter: Option<ObjectFilter>,
    pub after: Option<String>,
    pub first: Option<i32>,
}

// =============================================================================
//  Inner query fragments
// =============================================================================

/// `ObjectConnection` where the `Object` fragment does take any parameters.
#[derive(cynic::QueryFragment, Clone, Debug)]
#[cynic(graphql_type = "ObjectConnection")]
pub struct ObjectConnection<Object>
where
    Object: QueryFragment<SchemaType = schema::Object, VariablesFields = ()>,
{
    pub nodes: Vec<Object>,
    pub page_info: PageInfoForward,
}

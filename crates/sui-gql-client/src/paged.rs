use cynic::serde::de::DeserializeOwned;
use cynic::serde::Serialize;
use cynic::{GraphQlResponse, QueryFragment, QueryVariables};
use tap::TapFallible as _;

use crate::{GraphQlErrors, GraphQlResponseExt as _};

/// Pages resulting from GraphQL queries.
///
/// The full responses are included to allow users to introspect possible errors.
pub struct PagedResponse<Init>(
    pub(crate) GraphQlResponse<Init>,
    pub(crate) Vec<GraphQlResponse<Init::NextPage>>,
)
where
    Init: Paged;

impl<Init> PagedResponse<Init>
where
    Init: Paged,
{
    /// Extract data (inital query, subsequent queries) from the [GraphQlResponse]s.
    ///
    /// Errors if any response has errors.
    pub fn try_into_data(self) -> PagesDataResult<Init> {
        let Self(first, next) = self;

        let Some(initial) = first.try_into_data().tap_err_mut(|e| e.page = Some(0))? else {
            return Ok(None);
        };

        let mut pages = vec![];
        for (i, response) in next.into_iter().enumerate() {
            if let Some(page_data) = response
                .try_into_data()
                .tap_err_mut(|e| e.page = Some(i + 1))?
            {
                pages.push(page_data);
            } else {
                break;
            }
        }

        Ok(Some((initial, pages)))
    }

    pub fn into_inner(self) -> (GraphQlResponse<Init>, Vec<GraphQlResponse<Init::NextPage>>) {
        (self.0, self.1)
    }
}

/// The initial page data and subsequent ones', if any.
pub type PagesDataResult<T> = Result<Option<(T, Vec<<T as Paged>::NextPage>)>, GraphQlErrors>;

/// Interface for paged queries to allow automatic pagination.
pub trait Paged:
    DeserializeOwned + QueryFragment<VariablesFields = <Self::Input as QueryVariables>::Fields>
{
    /// The input for this query type.
    type Input: QueryVariables + Send + Serialize;
    /// The type of the next query variables.
    type NextInput: QueryVariables + Send + Serialize;
    /// The type of the next query.
    type NextPage: DeserializeOwned
        + QueryFragment<VariablesFields = <Self::NextInput as QueryVariables>::Fields>
        + Send;

    /// The next query variables to use for querying the next page, if any.
    fn next_variables(&self, prev_vars: Self::Input) -> Option<Self::NextInput>;
}

//! Experimental abstractions to help create stream queries.
//!
//! Should be private until we iron out the api.
use std::future::Future;

use futures::Stream;

use super::model::fragments::PageInfo;
use crate::GraphQlClient;

/// Helper for paginating queries forward.
///
/// # Arguments
/// - `client`
/// - `vars`: [`cynic::QueryVariables`] fragment; must implement [`UpdatePageInfo`]
/// - `request`: async function that maps `(client, vars) -> Page<Iter>`, where `Iter` is an
///   iterator over items of a **single** page's results
pub(super) fn forward<'a, Client, Vars, Req, Fut, Iter, T, Err>(
    client: &'a Client,
    mut vars: Vars,
    mut request: Req,
) -> impl Stream<Item = Result<T, Err>> + 'a
where
    Client: GraphQlClient,
    Vars: 'a + UpdatePageInfo + Clone,
    Req: 'a + FnMut(&'a Client, Vars) -> Fut,
    Fut: Future<Output = Result<Page<Iter>, Err>>,
    Iter: IntoIterator<Item = Result<T, Err>>,
    T: 'static,
    Err: 'a,
{
    async_stream::try_stream! {
        let mut has_next_page = true;
        while has_next_page {
            let page = request(client, vars.clone()).await?;

            vars.update_page_info(&page.info);
            has_next_page = page.info.has_next_page;

            for value in page.data {
                yield value?;
            }
        }
    }
}

pub(super) struct Page<T> {
    pub(super) info: PageInfo,
    pub(super) data: T,
}

impl<T> Page<T> {
    pub(super) fn new(page_info: impl Into<PageInfo>, data: T) -> Self {
        Self {
            info: page_info.into(),
            data,
        }
    }
}

pub(super) trait UpdatePageInfo {
    fn update_page_info(&mut self, info: &PageInfo);
}

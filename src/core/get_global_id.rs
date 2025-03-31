use super::uprotocol_pkg::{BasePkg, GetBaseInfo};

pub trait GetGlobalId {
    fn get_global_id(&self) -> String;
}

impl GetGlobalId for BasePkg {
    fn get_global_id(&self) -> String {
        crate::utils::get_global_id(&self.client_class, &self.client_instance)
    }
}

impl<T> GetGlobalId for T
where
    T: GetBaseInfo,
{
    fn get_global_id(&self) -> String {
        self.get_baseinfo().get_global_id()
    }
    
}
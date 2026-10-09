use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Eq)]
pub struct Request {
    pub id: String,
    pub body: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ServiceError {
    EmptyId,
    EmptyBody,
    DuplicateId,
}

#[derive(Default)]
pub struct Service {
    records: BTreeMap<String, String>,
}

impl Service {
    pub fn create(&mut self, request: Request) -> Result<&str, ServiceError> {
        let id = request.id.trim();
        let body = request.body.trim();
        if id.is_empty() {
            return Err(ServiceError::EmptyId);
        }
        if body.is_empty() {
            return Err(ServiceError::EmptyBody);
        }
        if self.records.contains_key(id) {
            return Err(ServiceError::DuplicateId);
        }
        self.records.insert(id.to_owned(), body.to_owned());
        Ok(self.records.get_key_value(id).expect("inserted record").0)
    }

    pub fn get(&self, id: &str) -> Option<&str> {
        self.records.get(id).map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::{Request, Service, ServiceError};

    #[test]
    fn create_then_read_returns_normalized_record() {
        let mut service = Service::default();
        let id = service
            .create(Request {
                id: " task-1 ".into(),
                body: "  prepare release  ".into(),
            })
            .unwrap();
        assert_eq!(id, "task-1");
        assert_eq!(service.get("task-1"), Some("prepare release"));
    }

    #[test]
    fn rejects_empty_fields_and_duplicate_ids() {
        let mut service = Service::default();
        assert_eq!(
            service.create(Request {
                id: " ".into(),
                body: "body".into(),
            }),
            Err(ServiceError::EmptyId)
        );
        assert_eq!(
            service.create(Request {
                id: "1".into(),
                body: " ".into(),
            }),
            Err(ServiceError::EmptyBody)
        );
        service
            .create(Request {
                id: "1".into(),
                body: "body".into(),
            })
            .unwrap();
        assert_eq!(
            service.create(Request {
                id: "1".into(),
                body: "another".into(),
            }),
            Err(ServiceError::DuplicateId)
        );
    }
}

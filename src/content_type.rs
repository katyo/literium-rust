use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct ContentType<'ct> {
    fulltype: Cow<'ct, str>,
    typeoffs: Vec<usize>,
}

impl<'ct1, 'ct2> PartialEq<ContentType<'ct2>> for ContentType<'ct1> {
    fn eq(&self, other: &ContentType<'ct2>) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl<'ct> AsRef<str> for ContentType<'ct> {
    fn as_ref(&self) -> &str {
        &self.fulltype[0..self.typeoffs[self.typeoffs.len() - 1]]
    }
}

impl<'ct> ContentType<'ct> {
    pub fn new<S>(src: S) -> Self
    where
        S: Into<Cow<'ct, str>>,
    {
        let fulltype = src.into();
        let mut typeoffs = Vec::new();
        {
            let src = fulltype.as_ref();
            if let Some(maintype_end) = src.find('/') {
                typeoffs.push(maintype_end);
                let mut off = maintype_end + 1;
                loop {
                    if let Some(subtype_end) = src[off..].find('+') {
                        off += subtype_end;
                        typeoffs.push(off);
                        off += 1;
                    } else {
                        typeoffs.push(src.len());
                        break;
                    }
                }
            } else {
                typeoffs.push(src.len());
            }
        }
        ContentType { fulltype, typeoffs }
    }

    pub fn get_type(&self) -> &str {
        &self.fulltype.as_ref()[0..self.typeoffs[0]]
    }

    pub fn num_subtypes(&self) -> usize {
        self.typeoffs.len() - 1
    }

    pub fn get_subtype(&self, index: usize) -> Option<&str> {
        let len = self.num_subtypes();
        if index < len {
            let fulltype = self.fulltype.as_ref();
            Some(&fulltype[self.typeoffs[index] + 1..self.typeoffs[index + 1]])
        } else {
            None
        }
    }

    pub fn last_subtype(&self) -> Option<&str> {
        self.get_subtype(self.num_subtypes() - 1)
    }

    pub fn pop_subtype(&mut self) {
        if self.num_subtypes() > 0 {
            self.typeoffs.pop();
        }
    }

    pub fn push_subtype(&mut self, subtype: &str) {
        let has_subtypes = self.num_subtypes() > 0;
        let fulltype = self.fulltype.to_mut();
        fulltype.push(if has_subtypes { '+' } else { '/' });
        fulltype.push_str(subtype);
        self.typeoffs.push(fulltype.len());
    }

    pub fn iter_subtypes(&'ct self) -> ContentTypeSubtypesIterator<'ct> {
        ContentTypeSubtypesIterator {
            content_type: &self,
            subtype_index: 0,
        }
    }
}

pub struct ContentTypeSubtypesIterator<'ct> {
    content_type: &'ct ContentType<'ct>,
    subtype_index: usize,
}

impl<'ct> Iterator for ContentTypeSubtypesIterator<'ct> {
    type Item = &'ct str;
    fn next(&mut self) -> Option<Self::Item> {
        if self.subtype_index < self.content_type.num_subtypes() {
            let subtype_index = self.subtype_index;
            self.subtype_index += 1;
            self.content_type.get_subtype(subtype_index)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_type() {
        assert_eq!(
            ContentType::new("application/json").get_type(),
            "application"
        );
    }

    #[test]
    fn test_get_type_no_subtypes() {
        assert_eq!(
            ContentType::new("application-json").get_type(),
            "application-json"
        );
    }

    #[test]
    fn test_get_subtype() {
        let ct = ContentType::new("application/json");
        assert_eq!(ct.num_subtypes(), 1);
        assert_eq!(ct.get_subtype(0), Some("json"));
    }

    #[test]
    fn test_get_subtype_no_subtypes() {
        let ct = ContentType::new("application-json");
        assert_eq!(ct.num_subtypes(), 0);
        assert_eq!(ct.get_subtype(0), None);
    }

    #[test]
    fn test_get_subtype_two_subtypes() {
        let ct = ContentType::new("application/json+base64");
        assert_eq!(ct.num_subtypes(), 2);
        assert_eq!(ct.get_subtype(0), Some("json"));
        assert_eq!(ct.get_subtype(1), Some("base64"));
    }

    #[test]
    fn test_get_subtype_three_subtypes() {
        let ct = ContentType::new("application/json+sbox+base64");
        assert_eq!(ct.num_subtypes(), 3);
        assert_eq!(ct.get_subtype(0), Some("json"));
        assert_eq!(ct.get_subtype(1), Some("sbox"));
        assert_eq!(ct.get_subtype(2), Some("base64"));
    }

    #[test]
    fn test_iter_subtypes_three_subtypes() {
        let ct = ContentType::new("application/json+sbox+base64");
        let mut cti = ct.iter_subtypes();
        assert_eq!(cti.next(), Some("json"));
        assert_eq!(cti.next(), Some("sbox"));
        assert_eq!(cti.next(), Some("base64"));
        assert_eq!(cti.next(), None);
    }

    #[test]
    fn test_pop_subtype() {
        let mut ct = ContentType::new("application/json+sbox+base64");
        ct.pop_subtype();
        assert_eq!(ct.as_ref(), "application/json+sbox");
        ct.pop_subtype();
        assert_eq!(ct.as_ref(), "application/json");
        ct.pop_subtype();
        assert_eq!(ct.as_ref(), "application");
    }

    #[test]
    fn test_push_subtype() {
        let mut ct = ContentType::new("application");
        assert_eq!(ct.as_ref(), "application");
        ct.push_subtype("json");
        assert_eq!(ct.as_ref(), "application/json");
        ct.push_subtype("sbox");
        assert_eq!(ct.as_ref(), "application/json+sbox");
        ct.push_subtype("base64");
        assert_eq!(ct.as_ref(), "application/json+sbox+base64");
    }
}

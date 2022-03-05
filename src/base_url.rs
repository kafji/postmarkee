use crate::Error;
use url::Url;

#[derive(PartialEq, Clone, Debug)]
pub struct BaseUrl(Url);

impl BaseUrl {
    pub fn as_url(&self) -> &Url {
        &self.0
    }

    pub fn into_inner(self) -> Url {
        self.0
    }
}

impl TryFrom<Url> for BaseUrl {
    type Error = Error;

    fn try_from(value: Url) -> Result<Self, Self::Error> {
        if value.cannot_be_a_base() {
            return Err(Error::Url {
                actual: value,
                reason: "expecting a base URL".to_owned(),
            });
        }
        if !["https", "http"].contains(&value.scheme()) {
            return Err(Error::Url {
                actual: value,
                reason: "expecting an HTTP URL".to_owned(),
            });
        }
        Ok(Self(value))
    }
}

impl Default for BaseUrl {
    fn default() -> Self {
        let url = Url::parse("https://api.postmarkapp.com").unwrap();
        let base_url = url.try_into().unwrap();
        Self(base_url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let url = BaseUrl::default().as_url().to_string();
        assert_eq!(url, "https://api.postmarkapp.com/");
    }

    #[test]
    fn test_not_http_url() {
        let url: Url = "wss://example.com".parse().unwrap();
        let error = TryInto::<BaseUrl>::try_into(url).unwrap_err();
        assert_eq!(
            error.to_string(),
            "expecting an HTTP URL, was `wss://example.com/`"
        );
    }

    #[test]
    fn test_not_base_url() {
        let url: Url = "data:text/plain,Stuff".parse().unwrap();
        let error = TryInto::<BaseUrl>::try_into(url).unwrap_err();
        assert_eq!(
            error.to_string(),
            "expecting a base URL, was `data:text/plain,Stuff`"
        );
    }
}

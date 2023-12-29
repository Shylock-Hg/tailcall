use crate::blueprint::*;
use crate::config;
use crate::config::{Config, Field};
use crate::lambda::{Cache, Expression};
use crate::try_fold::TryFold;
use crate::valid::Valid;

pub fn update_cache<'a>() -> TryFold<'a, (&'a Config, &'a Field, &'a config::Type, &'a str), FieldDefinition, String> {
  TryFold::<(&Config, &Field, &config::Type, &str), FieldDefinition, String>::new(|(_config, field, _, _), b_field| {
    let mut updated_b_field = b_field;
    match updated_b_field.resolver.as_ref() {
      Some(source) => {
        if let Some(cache) = &field.cache {
          let cache = Expression::Cache(Cache::new(cache.max_age, Box::new(source.clone())));
          updated_b_field.resolver = Some(cache);
        }
        Valid::succeed(updated_b_field)
      }
      None => Valid::succeed(updated_b_field),
    }
  })
}
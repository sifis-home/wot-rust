use serde::{ser::SerializeStruct, Deserialize, Deserializer, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Nil;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cons<T, U = Nil> {
    #[serde(flatten)]
    pub(crate) head: T,
    #[serde(flatten)]
    pub(crate) tail: U,
}

impl<T> Cons<T, Nil> {
    #[inline]
    pub(crate) fn new_head(head: T) -> Self {
        Cons { head, tail: Nil {} }
    }
}

trait ToRef<'a> {
    type Ref: 'a;
    type Mut: 'a;
    /// Return a heterogenous list of references
    fn to_ref(&'a self) -> Self::Ref;
    /// Return a heterogenous list of mutable references
    fn to_mut(&'a mut self) -> Self::Mut;
}

impl<'a> ToRef<'a> for Nil {
    type Ref = &'a Nil;
    type Mut = &'a mut Nil;
    /// Return a heterogenous list of references
    fn to_ref(&'a self) -> Self::Ref {
        self
    }
    /// Return a heterogenous list of mutable references
    fn to_mut(&'a mut self) -> Self::Mut {
        self
    }
}

impl<'a, T, U> ToRef<'a> for Cons<T, U>
where
    T: 'a,
    U: ToRef<'a>,
{
    type Ref = Cons<&'a T, U::Ref>;
    type Mut = Cons<&'a mut T, U::Mut>;

    /// Return a heterogenous list of references
    fn to_ref(&'a self) -> Self::Ref {
        let Cons { ref head, ref tail } = self;

        Cons {
            head,
            tail: tail.to_ref(),
        }
    }

    /// Return a heterogenous list of mutable references
    fn to_mut(&'a mut self) -> Self::Mut {
        let Cons {
            ref mut head,
            ref mut tail,
        } = self;

        Cons {
            head,
            tail: tail.to_mut(),
        }
    }
}

impl<T, U> Cons<T, U> {
    #[inline]
    pub(crate) fn add<V>(self, value: V) -> Cons<V, Self> {
        Cons {
            head: value,
            tail: self,
        }
    }

    /// Split the head element of the heterogenous list
    pub fn split_head(self) -> (T, U) {
        let Cons { head, tail } = self;

        (head, tail)
    }
}

impl Serialize for Nil {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_struct("Nil", 0)?.end()
    }
}

impl<'de> Deserialize<'de> for Nil {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct NilStruct {}

        NilStruct::deserialize(deserializer)?;
        Ok(Nil)
    }
}

impl From<()> for Nil {
    fn from(_: ()) -> Self {
        Nil
    }
}

impl From<Nil> for () {
    fn from(_: Nil) -> Self {}
}

#[cfg(test)]
mod tests {
    use serde_json::{json, Value};

    use super::*;

    #[test]
    fn split_head() {
        #[derive(Debug, PartialEq)]
        struct A(i32);

        #[derive(Debug, PartialEq)]
        struct B(f32);

        #[derive(Debug, PartialEq)]
        struct C(String);

        let list = Cons::new_head(A(42)).add(B(1.234)).add(C("C".to_string()));

        assert_eq!(
            list.split_head(),
            (C("C".to_string()), Cons::new_head(A(42)).add(B(1.234))),
        )
    }

    #[test]
    fn chain() {
        let list = Cons::new_head("A").add(2).add("C".to_string());

        assert_eq!(
            list,
            Cons {
                head: "C".to_string(),
                tail: Cons {
                    head: 2,
                    tail: Cons {
                        head: "A",
                        tail: Nil,
                    },
                }
            }
        );
    }

    #[test]
    fn serialize_flatten_nil() {
        #[derive(Debug, Serialize)]
        struct A {
            a: i32,
            #[serde(flatten)]
            b: Nil,
        }

        let value = serde_json::to_value(A { a: 42, b: Nil {} }).unwrap();
        assert_eq!(value.get("a").unwrap(), &Value::Number(42.into()));
        assert!(value.get("b").is_none());
    }

    #[test]
    fn serialize_cons() {
        #[derive(Debug, Serialize)]
        struct C {
            bar: &'static str,
        }
        #[derive(Debug, Serialize)]
        struct B {
            foo: usize,
        }
        #[derive(Debug, Serialize)]
        struct A {
            a: i32,
            #[serde(flatten)]
            b: Cons<C, Cons<B, Nil>>,
        }

        let value = serde_json::to_value(A {
            a: 42,
            b: Cons::new_head(B { foo: 42 }).add(C { bar: "42" }),
        })
        .unwrap();
        assert_eq!(value.get("a").unwrap(), &Value::Number(42.into()));
        assert_eq!(value.get("foo").unwrap(), &Value::Number(42.into()));
    }

    #[test]
    fn deserialize_cons() {
        #[derive(Debug, Deserialize)]
        struct C {
            bar: String,
        }
        #[derive(Debug, Deserialize)]
        struct B {
            foo: usize,
        }
        #[derive(Debug, Deserialize)]
        struct A {
            a: i32,
            #[serde(flatten)]
            b: Cons<Cons<B, C>, Nil>,
        }

        let v = json!({
            "a": 42,
            "foo": 42,
            "bar": "42",
        });

        let a: A = serde_json::from_value(v).unwrap();

        assert_eq!(a.a, 42);
        assert_eq!(a.b.head.head.foo, 42);
        assert_eq!(a.b.head.tail.bar, String::from("42"));
    }
}

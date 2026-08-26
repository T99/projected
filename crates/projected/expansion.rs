mod optional {
    use projected::*;
    #[projected_internal(
        projections(
            PartialOptional(include(name), optional(age)),
            OptionalFields(optional(name, age))
        )
    )]
    struct MyStruct {
        pub name: String,
        pub age: u32,
        pub email: String,
    }
    /// Backend-neutral logical fields generated for this model.
    enum MyStructField {
        ///Logical field identity for `name`.
        Name,
        ///Logical field identity for `age`.
        Age,
        ///Logical field identity for `email`.
        Email,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for MyStructField {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    MyStructField::Name => "Name",
                    MyStructField::Age => "Age",
                    MyStructField::Email => "Email",
                },
            )
        }
    }
    #[automatically_derived]
    #[doc(hidden)]
    unsafe impl ::core::clone::TrivialClone for MyStructField {}
    #[automatically_derived]
    impl ::core::clone::Clone for MyStructField {
        #[inline]
        fn clone(&self) -> MyStructField {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for MyStructField {}
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for MyStructField {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for MyStructField {
        #[inline]
        fn eq(&self, other: &MyStructField) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for MyStructField {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {}
    }
    #[automatically_derived]
    impl ::core::hash::Hash for MyStructField {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            ::core::hash::Hash::hash(&__self_discr, state)
        }
    }
    impl ::projected::ProjectedField for MyStructField {
        fn metadata(self) -> ::projected::FieldMetadata {
            match self {
                Self::Name => {
                    ::projected::FieldMetadata::new("name", "name", "name", true)
                }
                Self::Age => ::projected::FieldMetadata::new("age", "age", "age", true),
                Self::Email => {
                    ::projected::FieldMetadata::new("email", "email", "email", true)
                }
            }
        }
    }
    impl ::projected::ProjectedModel for MyStruct {
        type Field = MyStructField;
        fn fields() -> &'static [Self::Field] {
            &[MyStructField::Name, MyStructField::Age, MyStructField::Email]
        }
    }
    impl ::projected::ProjectedFieldMapping<MyStruct> for MyStructField {
        fn source_field(self) -> <MyStruct as ::projected::ProjectedModel>::Field {
            match self {
                Self::Name => MyStructField::Name,
                Self::Age => MyStructField::Age,
                Self::Email => MyStructField::Email,
            }
        }
    }
    ///Projection of [`MyStruct`].
    struct PartialOptional {
        ///Projected value of the source `name` field.
        pub name: String,
        ///Projected value of the source `age` field.
        pub age: ::core::option::Option<u32>,
    }
    ///Values required to complete [`PartialOptional`] into [`MyStruct`].
    struct PartialOptionalMissing {
        ///Value used for the source `age` field when completing the projection.
        pub age: u32,
        ///Value used for the source `email` field when completing the projection.
        pub email: String,
    }
    /// Backend-neutral logical fields generated for this model.
    enum PartialOptionalField {
        ///Logical field identity for `name`.
        Name,
        ///Logical field identity for `age`.
        Age,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for PartialOptionalField {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    PartialOptionalField::Name => "Name",
                    PartialOptionalField::Age => "Age",
                },
            )
        }
    }
    #[automatically_derived]
    #[doc(hidden)]
    unsafe impl ::core::clone::TrivialClone for PartialOptionalField {}
    #[automatically_derived]
    impl ::core::clone::Clone for PartialOptionalField {
        #[inline]
        fn clone(&self) -> PartialOptionalField {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for PartialOptionalField {}
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for PartialOptionalField {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for PartialOptionalField {
        #[inline]
        fn eq(&self, other: &PartialOptionalField) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for PartialOptionalField {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {}
    }
    #[automatically_derived]
    impl ::core::hash::Hash for PartialOptionalField {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            ::core::hash::Hash::hash(&__self_discr, state)
        }
    }
    impl ::projected::ProjectedField for PartialOptionalField {
        fn metadata(self) -> ::projected::FieldMetadata {
            match self {
                Self::Name => {
                    ::projected::FieldMetadata::new("name", "name", "name", true)
                }
                Self::Age => ::projected::FieldMetadata::new("age", "age", "age", true),
            }
        }
    }
    impl ::projected::ProjectedModel for PartialOptional {
        type Field = PartialOptionalField;
        fn fields() -> &'static [Self::Field] {
            &[PartialOptionalField::Name, PartialOptionalField::Age]
        }
    }
    impl ::projected::ProjectedFieldMapping<MyStruct> for PartialOptionalField {
        fn source_field(self) -> <MyStruct as ::projected::ProjectedModel>::Field {
            match self {
                Self::Name => MyStructField::Name,
                Self::Age => MyStructField::Age,
            }
        }
    }
    impl ::projected::Projection for PartialOptional {
        type Base = MyStruct;
        type Missing = PartialOptionalMissing;
        fn complete(self, missing: Self::Missing) -> Self::Base {
            MyStruct {
                name: self.name,
                age: match self.age {
                    ::core::option::Option::Some(value) => value,
                    ::core::option::Option::None => missing.age,
                },
                email: missing.email,
            }
        }
    }
    impl ::core::convert::From<MyStruct> for PartialOptional {
        fn from(base: MyStruct) -> Self {
            Self {
                name: base.name,
                age: ::core::option::Option::Some(base.age),
            }
        }
    }
    impl PartialOptional {
        ///Completes [`PartialOptional`] from a source-ordered list of omitted values and fallbacks.
        pub fn complete_with(self, age_fallback: u32, email: String) -> MyStruct {
            <Self as ::projected::Projection>::complete(
                self,
                PartialOptionalMissing {
                    age: age_fallback,
                    email: email,
                },
            )
        }
        /// Completes this projection from its generated missing-values representation.
        pub fn complete(self, missing: PartialOptionalMissing) -> MyStruct {
            <Self as ::projected::Projection>::complete(self, missing)
        }
    }
    ///Projection of [`MyStruct`].
    struct OptionalFields {
        ///Projected value of the source `name` field.
        pub name: ::core::option::Option<String>,
        ///Projected value of the source `age` field.
        pub age: ::core::option::Option<u32>,
        ///Projected value of the source `email` field.
        pub email: String,
    }
    ///Values required to complete [`OptionalFields`] into [`MyStruct`].
    struct OptionalFieldsMissing {
        ///Value used for the source `name` field when completing the projection.
        pub name: String,
        ///Value used for the source `age` field when completing the projection.
        pub age: u32,
    }
    /// Backend-neutral logical fields generated for this model.
    enum OptionalFieldsField {
        ///Logical field identity for `name`.
        Name,
        ///Logical field identity for `age`.
        Age,
        ///Logical field identity for `email`.
        Email,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for OptionalFieldsField {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    OptionalFieldsField::Name => "Name",
                    OptionalFieldsField::Age => "Age",
                    OptionalFieldsField::Email => "Email",
                },
            )
        }
    }
    #[automatically_derived]
    #[doc(hidden)]
    unsafe impl ::core::clone::TrivialClone for OptionalFieldsField {}
    #[automatically_derived]
    impl ::core::clone::Clone for OptionalFieldsField {
        #[inline]
        fn clone(&self) -> OptionalFieldsField {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for OptionalFieldsField {}
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for OptionalFieldsField {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for OptionalFieldsField {
        #[inline]
        fn eq(&self, other: &OptionalFieldsField) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for OptionalFieldsField {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {}
    }
    #[automatically_derived]
    impl ::core::hash::Hash for OptionalFieldsField {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            ::core::hash::Hash::hash(&__self_discr, state)
        }
    }
    impl ::projected::ProjectedField for OptionalFieldsField {
        fn metadata(self) -> ::projected::FieldMetadata {
            match self {
                Self::Name => {
                    ::projected::FieldMetadata::new("name", "name", "name", true)
                }
                Self::Age => ::projected::FieldMetadata::new("age", "age", "age", true),
                Self::Email => {
                    ::projected::FieldMetadata::new("email", "email", "email", true)
                }
            }
        }
    }
    impl ::projected::ProjectedModel for OptionalFields {
        type Field = OptionalFieldsField;
        fn fields() -> &'static [Self::Field] {
            &[
                OptionalFieldsField::Name,
                OptionalFieldsField::Age,
                OptionalFieldsField::Email,
            ]
        }
    }
    impl ::projected::ProjectedFieldMapping<MyStruct> for OptionalFieldsField {
        fn source_field(self) -> <MyStruct as ::projected::ProjectedModel>::Field {
            match self {
                Self::Name => MyStructField::Name,
                Self::Age => MyStructField::Age,
                Self::Email => MyStructField::Email,
            }
        }
    }
    impl ::projected::Projection for OptionalFields {
        type Base = MyStruct;
        type Missing = OptionalFieldsMissing;
        fn complete(self, missing: Self::Missing) -> Self::Base {
            MyStruct {
                name: match self.name {
                    ::core::option::Option::Some(value) => value,
                    ::core::option::Option::None => missing.name,
                },
                age: match self.age {
                    ::core::option::Option::Some(value) => value,
                    ::core::option::Option::None => missing.age,
                },
                email: self.email,
            }
        }
    }
    impl ::core::convert::From<MyStruct> for OptionalFields {
        fn from(base: MyStruct) -> Self {
            Self {
                name: ::core::option::Option::Some(base.name),
                age: ::core::option::Option::Some(base.age),
                email: base.email,
            }
        }
    }
    impl OptionalFields {
        ///Completes [`OptionalFields`] from a source-ordered list of omitted values and fallbacks.
        pub fn complete_with(
            self,
            name_fallback: String,
            age_fallback: u32,
        ) -> MyStruct {
            <Self as ::projected::Projection>::complete(
                self,
                OptionalFieldsMissing {
                    name: name_fallback,
                    age: age_fallback,
                },
            )
        }
        /// Completes this projection from its generated missing-values representation.
        pub fn complete(self, missing: OptionalFieldsMissing) -> MyStruct {
            <Self as ::projected::Projection>::complete(self, missing)
        }
    }
    fn asd() {
        let a = MyStruct {
            name: "Alice".to_string(),
            age: 30,
            email: "alice@example.com".to_string(),
        };
    }
}

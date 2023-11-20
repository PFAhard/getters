# Rust Getters Derive Macro Library

This Rust library provides a powerful and easy-to-use procedural macro `derive_getters_fn` for automatically deriving getter methods for struct fields. Designed to streamline the process of creating getters in Rust structs, this library enhances code clarity and development efficiency.

# Features

- Automatic Getter Generation: Simplifies the process of creating getters for each field in a Rust struct. This feature is particularly useful in large structs or when working with complex data structures.

- Customizable Through Attributes: Offers a range of attributes to customize the behavior of the generated getter methods. Attributes like `use_deref`, `use_as_ref`, `get_mut`, `skip_new`, `getter_logic`, and `skip_getter` allow developers to tailor the getters to specific requirements.

- Support for Various Field Types: Whether your struct has named or unnamed fields (such as in tuples), the macro can handle them efficiently, ensuring that appropriate getters are generated for each scenario.

- Mutable Getters: In addition to standard immutable getters, the library supports the generation of mutable getters with the get_mut attribute, providing greater flexibility.

- Custom Logic for Getters: The `getter_logic` attribute allows the integration of custom logic into the getter methods, offering the ability to have more complex getters beyond simple field access.

- Optional Constructor Generation: With the `skip_new` attribute, users can choose to generate a constructor method (new) for the struct. This is particularly useful for ensuring struct integrity upon instantiation.

# Usage

The library is designed for ease of use. After including it in your project, simply annotate your struct with `#[derive(Getters)]`, and use the provided attributes to customize the getter generation as needed.

# Target Audience

This library is ideal for Rust developers who regularly work with structs and require an efficient way to generate getters. It is especially useful in applications where data encapsulation and object-oriented patterns are prevalent.

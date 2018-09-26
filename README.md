# diesel-setup-deps
Perform diesel setup for dependencies


## Instructions for library authors

- Create a file named `EXPORT_MIGRATIONS` in the same location as your `Cargo.toml` file.
- This file should contain the relative path to your migrations directory.
- Make sure your migrations are included in published version of your crate.


## Instructions for library users

- Install this crate using `cargo install diesel-setup-deps`
- Ensure `diesel setup` has been run previously.
- Run `diesel-setup-deps`.

Migrations from your dependencies will now automatically be added to your migrations directory.

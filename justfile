# default, list all just Recipe
default: 
  @just -q --list

alias t := test
alias l := clippy

set export

# cargo get: https://crates.io/crates/cargo-get
env_version := `cargo get --entry="./Cargo.toml" package.version --pretty`
env_version_msg := "New release with version: " + env_version


# run all tests with all-features
test filter="":
  @cargo test --all-features {{filter}}

# cargo watch for test with given filter
watch filter="":
  @cargo watch -q -c -x 'test {{filter}}'

# run cargo test and clippy
clippy: test
  @cargo clippy --tests --workspace -- -D warnings

# run cargo doc --no-deps
doc:
  @cargo doc --no-deps


# generate the README.md from the README.tpl + src/lib.rs
readme:
  cargo readme -o ./README.md 

echo_version:
  @echo "Version: " $env_version

release_new_version:
  @echo "$env_version_msg"
  git tag -a "$env_version" -m "$env_version_msg"
  git push origin --tags
  cargo publish -v

// https://github.com/rust-lang/rfcs/blob/master/text/2585-unsafe-block-in-unsafe-fn.md
#![deny(unsafe_op_in_unsafe_fn)]

use std::path::Path;

pub struct ShadowBuildCommon {
    deps: Option<system_deps::Dependencies>,
    build_src_root: Box<Path>,
    src_root: Box<Path>,
}

impl ShadowBuildCommon {
    pub fn new(repo_root: &Path, system_deps: Option<system_deps::Dependencies>) -> Self {
        let src_root = {
            let mut p = repo_root.to_path_buf();
            p.push("src");
            p.into_boxed_path()
        };

        let build_src_root = {
            let mut p = repo_root.to_path_buf();
            p.push("build");
            p.push("src");
            p.into_boxed_path()
        };

        // Conservatively re-run build scripts if anything in their package directory
        // changes.
        println!("cargo:rerun-if-changed=.");

        Self {
            deps: system_deps,
            build_src_root,
            src_root,
        }
    }

    pub fn cc_build(&self) -> cc::Build {
        let mut b = cc::Build::new();
        println!("cargo:rerun-if-env-changed=CC");
        println!("cargo:rerun-if-env-changed=CXX");
        println!("cargo:rerun-if-env-changed=CFLAGS");
        println!("cargo:rerun-if-env-changed=CXXFLAGS");

        // When adding flags here, consider using `add_compile_options`
        // in the root CMakeLists.txt instead, where they will be picked
        // up both here and in our remaining pure C targets.
        b.define("_GNU_SOURCE", None)
            .include(&*self.build_src_root)
            .include(&*self.src_root)
            // Disable extra warnings (-Wall, -Wextra) until if and when they're
            // fixed in our C code.
            .warnings(false)
            // By default, *don't* convert any remaining warnings into errors (-Werror).
            // -Werror is currently enabled here via CFLAGS, which
            // cmake sets depending on the option SHADOW_WERROR.
            .warnings_into_errors(false);

        if let Some(deps) = &self.deps {
            b.includes(deps.all_include_paths());
        }

        if let Some("true") = std::env::var("DEBUG").ok().as_deref() {
            b.flag("-DDEBUG")
                // we only check for unused functions when builing in debug mode since some
                // functions are only called when logging, which can be #ifdef'd out in
                // release mode
                .flag("-Wunused-function");
        } else {
            b.flag("-DNDEBUG");
        }

        b
    }

    #[cfg(feature = "bindgen")]
    pub fn bindgen_builder(&self) -> bindgen::Builder {
        let mut builder = bindgen::Builder::default()
            // Tell cargo to invalidate the built crate whenever any of the
            // included header files changed.
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
            .clang_args(&[
                &format!("-I{}", self.build_src_root.to_str().unwrap()),
                &format!("-I{}", self.src_root.to_str().unwrap()),
                "-D_GNU_SOURCE",
            ])
            //# used to generate #[must_use] annotations)
            .enable_function_attribute_detection();

        if let Some(deps) = &self.deps {
            for path in deps.all_include_paths() {
                builder = builder.clang_args(&[format!("-I{}", path.to_str().unwrap())]);
            }
        }
        builder
    }

    #[cfg(feature = "cbindgen")]
    pub fn cbindgen_base_config(&self) -> cbindgen::Config {
        let header = "
/*
 * The Shadow Simulator
 * See LICENSE for licensing information
 */
// clang-format off";
        let mut config = cbindgen::Config::default();
        config.language = cbindgen::Language::C;
        config.line_length = 100;
        config.documentation_style = cbindgen::DocumentationStyle::C99;
        config.macro_expansion.bitflags = true;
        config.header = Some(header.into());
        config.autogen_warning = Some(
            "/* Warning, this file is autogenerated by cbindgen. Don't modify this manually. */"
                .into(),
        );
        config.enumeration = cbindgen::EnumConfig {
            prefix_with_name: true,
            rename_variants: cbindgen::RenameRule::ScreamingSnakeCase,
            ..cbindgen::EnumConfig::default()
        };
        config.function = cbindgen::FunctionConfig {
            must_use: Some("__attribute__((warn_unused_result))".into()),
            no_return: Some("__attribute__((noreturn))".into()),
            ..cbindgen::FunctionConfig::default()
        };
        config.export = cbindgen::ExportConfig {
            rename: std::collections::HashMap::from([
                ("timeval".into(), "struct timeval".into()),
                ("timespec".into(), "struct timespec".into()),
            ]),
            // All types.
            item_types: vec![
                cbindgen::ItemType::Enums,
                cbindgen::ItemType::Constants,
                cbindgen::ItemType::Globals,
                cbindgen::ItemType::Structs,
                cbindgen::ItemType::Unions,
                cbindgen::ItemType::Typedefs,
                cbindgen::ItemType::OpaqueItems,
                cbindgen::ItemType::Functions,
            ],
            ..cbindgen::ExportConfig::default()
        };
        config
    }
}

#[cfg(feature = "cbindgen")]
pub trait CBindgenExt {
    fn get_mut(&mut self) -> &mut cbindgen::Config;

    // Export the given types opaquely.
    //
    // This overrides cbindgen's behavior of making any `repr(C)` type
    // non-opaque.
    // https://github.com/eqrion/cbindgen/issues/104
    fn add_opaque_types(&mut self, types: &[&str]) {
        let c = self.get_mut();
        if types.is_empty() {
            return;
        }
        if c.after_includes.is_none() {
            c.after_includes.replace("".into());
        }
        for t in types {
            c.after_includes
                .as_mut()
                .unwrap()
                .push_str(&format!("typedef struct {t} {t};\n"));
            c.export.exclude.push((*t).into());
        }
    }
}

#[cfg(feature = "cbindgen")]
impl CBindgenExt for cbindgen::Config {
    fn get_mut(&mut self) -> &mut cbindgen::Config {
        self
    }
}

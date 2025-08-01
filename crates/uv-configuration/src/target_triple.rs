use tracing::debug;

use uv_pep508::MarkerEnvironment;
use uv_platform_tags::{Arch, Os, Platform};
use uv_static::EnvVars;

/// The supported target triples. Each triple consists of an architecture, vendor, and operating
/// system.
///
/// See: <https://doc.rust-lang.org/nightly/rustc/platform-support.html>
#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum TargetTriple {
    /// An alias for `x86_64-pc-windows-msvc`, the default target for Windows.
    Windows,

    /// An alias for `x86_64-unknown-linux-gnu`, the default target for Linux.
    Linux,

    /// An alias for `aarch64-apple-darwin`, the default target for macOS.
    Macos,

    /// A 64-bit x86 Windows target.
    #[cfg_attr(feature = "clap", value(name = "x86_64-pc-windows-msvc"))]
    #[serde(rename = "x86_64-pc-windows-msvc")]
    #[serde(alias = "x8664-pc-windows-msvc")]
    X8664PcWindowsMsvc,

    /// A 32-bit x86 Windows target.
    #[cfg_attr(feature = "clap", value(name = "i686-pc-windows-msvc"))]
    #[serde(rename = "i686-pc-windows-msvc")]
    I686PcWindowsMsvc,

    /// An x86 Linux target. Equivalent to `x86_64-manylinux_2_28`.
    #[cfg_attr(feature = "clap", value(name = "x86_64-unknown-linux-gnu"))]
    #[serde(rename = "x86_64-unknown-linux-gnu")]
    #[serde(alias = "x8664-unknown-linux-gnu")]
    X8664UnknownLinuxGnu,

    /// An ARM-based macOS target, as seen on Apple Silicon devices
    ///
    /// By default, assumes the least-recent, non-EOL macOS version (13.0), but respects
    /// the `MACOSX_DEPLOYMENT_TARGET` environment variable if set.
    #[cfg_attr(feature = "clap", value(name = "aarch64-apple-darwin"))]
    #[serde(rename = "aarch64-apple-darwin")]
    Aarch64AppleDarwin,

    /// An x86 macOS target.
    ///
    /// By default, assumes the least-recent, non-EOL macOS version (13.0), but respects
    /// the `MACOSX_DEPLOYMENT_TARGET` environment variable if set.
    #[cfg_attr(feature = "clap", value(name = "x86_64-apple-darwin"))]
    #[serde(rename = "x86_64-apple-darwin")]
    #[serde(alias = "x8664-apple-darwin")]
    X8664AppleDarwin,

    /// An ARM64 Linux target. Equivalent to `aarch64-manylinux_2_28`.
    #[cfg_attr(feature = "clap", value(name = "aarch64-unknown-linux-gnu"))]
    #[serde(rename = "aarch64-unknown-linux-gnu")]
    Aarch64UnknownLinuxGnu,

    /// An ARM64 Linux target.
    #[cfg_attr(feature = "clap", value(name = "aarch64-unknown-linux-musl"))]
    #[serde(rename = "aarch64-unknown-linux-musl")]
    Aarch64UnknownLinuxMusl,

    /// An `x86_64` Linux target.
    #[cfg_attr(feature = "clap", value(name = "x86_64-unknown-linux-musl"))]
    #[serde(rename = "x86_64-unknown-linux-musl")]
    #[serde(alias = "x8664-unknown-linux-musl")]
    X8664UnknownLinuxMusl,

    /// An `x86_64` target for the `manylinux2014` platform. Equivalent to `x86_64-manylinux_2_17`.
    #[cfg_attr(feature = "clap", value(name = "x86_64-manylinux2014"))]
    #[serde(rename = "x86_64-manylinux2014")]
    #[serde(alias = "x8664-manylinux2014")]
    X8664Manylinux2014,

    /// An `x86_64` target for the `manylinux_2_17` platform.
    #[cfg_attr(feature = "clap", value(name = "x86_64-manylinux_2_17"))]
    #[serde(rename = "x86_64-manylinux_2_17")]
    #[serde(alias = "x8664-manylinux217")]
    X8664Manylinux217,

    /// An `x86_64` target for the `manylinux_2_28` platform.
    #[cfg_attr(feature = "clap", value(name = "x86_64-manylinux_2_28"))]
    #[serde(rename = "x86_64-manylinux_2_28")]
    #[serde(alias = "x8664-manylinux228")]
    X8664Manylinux228,

    /// An `x86_64` target for the `manylinux_2_31` platform.
    #[cfg_attr(feature = "clap", value(name = "x86_64-manylinux_2_31"))]
    #[serde(rename = "x86_64-manylinux_2_31")]
    #[serde(alias = "x8664-manylinux231")]
    X8664Manylinux231,

    /// An `x86_64` target for the `manylinux_2_32` platform.
    #[cfg_attr(feature = "clap", value(name = "x86_64-manylinux_2_32"))]
    #[serde(rename = "x86_64-manylinux_2_32")]
    #[serde(alias = "x8664-manylinux232")]
    X8664Manylinux232,

    /// An `x86_64` target for the `manylinux_2_33` platform.
    #[cfg_attr(feature = "clap", value(name = "x86_64-manylinux_2_33"))]
    #[serde(rename = "x86_64-manylinux_2_33")]
    #[serde(alias = "x8664-manylinux233")]
    X8664Manylinux233,

    /// An `x86_64` target for the `manylinux_2_34` platform.
    #[cfg_attr(feature = "clap", value(name = "x86_64-manylinux_2_34"))]
    #[serde(rename = "x86_64-manylinux_2_34")]
    #[serde(alias = "x8664-manylinux234")]
    X8664Manylinux234,

    /// An `x86_64` target for the `manylinux_2_35` platform.
    #[cfg_attr(feature = "clap", value(name = "x86_64-manylinux_2_35"))]
    #[serde(rename = "x86_64-manylinux_2_35")]
    #[serde(alias = "x8664-manylinux235")]
    X8664Manylinux235,

    /// An `x86_64` target for the `manylinux_2_36` platform.
    #[cfg_attr(feature = "clap", value(name = "x86_64-manylinux_2_36"))]
    #[serde(rename = "x86_64-manylinux_2_36")]
    #[serde(alias = "x8664-manylinux236")]
    X8664Manylinux236,

    /// An `x86_64` target for the `manylinux_2_37` platform.
    #[cfg_attr(feature = "clap", value(name = "x86_64-manylinux_2_37"))]
    #[serde(rename = "x86_64-manylinux_2_37")]
    #[serde(alias = "x8664-manylinux237")]
    X8664Manylinux237,

    /// An `x86_64` target for the `manylinux_2_38` platform.
    #[cfg_attr(feature = "clap", value(name = "x86_64-manylinux_2_38"))]
    #[serde(rename = "x86_64-manylinux_2_38")]
    #[serde(alias = "x8664-manylinux238")]
    X8664Manylinux238,

    /// An `x86_64` target for the `manylinux_2_39` platform.
    #[cfg_attr(feature = "clap", value(name = "x86_64-manylinux_2_39"))]
    #[serde(rename = "x86_64-manylinux_2_39")]
    #[serde(alias = "x8664-manylinux239")]
    X8664Manylinux239,

    /// An `x86_64` target for the `manylinux_2_40` platform.
    #[cfg_attr(feature = "clap", value(name = "x86_64-manylinux_2_40"))]
    #[serde(rename = "x86_64-manylinux_2_40")]
    #[serde(alias = "x8664-manylinux240")]
    X8664Manylinux240,

    /// An ARM64 target for the `manylinux2014` platform. Equivalent to `aarch64-manylinux_2_17`.
    #[cfg_attr(feature = "clap", value(name = "aarch64-manylinux2014"))]
    #[serde(rename = "aarch64-manylinux2014")]
    Aarch64Manylinux2014,

    /// An ARM64 target for the `manylinux_2_17` platform.
    #[cfg_attr(feature = "clap", value(name = "aarch64-manylinux_2_17"))]
    #[serde(rename = "aarch64-manylinux_2_17")]
    #[serde(alias = "aarch64-manylinux217")]
    Aarch64Manylinux217,

    /// An ARM64 target for the `manylinux_2_28` platform.
    #[cfg_attr(feature = "clap", value(name = "aarch64-manylinux_2_28"))]
    #[serde(rename = "aarch64-manylinux_2_28")]
    #[serde(alias = "aarch64-manylinux228")]
    Aarch64Manylinux228,

    /// An ARM64 target for the `manylinux_2_31` platform.
    #[cfg_attr(feature = "clap", value(name = "aarch64-manylinux_2_31"))]
    #[serde(rename = "aarch64-manylinux_2_31")]
    #[serde(alias = "aarch64-manylinux231")]
    Aarch64Manylinux231,

    /// An ARM64 target for the `manylinux_2_32` platform.
    #[cfg_attr(feature = "clap", value(name = "aarch64-manylinux_2_32"))]
    #[serde(rename = "aarch64-manylinux_2_32")]
    #[serde(alias = "aarch64-manylinux232")]
    Aarch64Manylinux232,

    /// An ARM64 target for the `manylinux_2_33` platform.
    #[cfg_attr(feature = "clap", value(name = "aarch64-manylinux_2_33"))]
    #[serde(rename = "aarch64-manylinux_2_33")]
    #[serde(alias = "aarch64-manylinux233")]
    Aarch64Manylinux233,

    /// An ARM64 target for the `manylinux_2_34` platform.
    #[cfg_attr(feature = "clap", value(name = "aarch64-manylinux_2_34"))]
    #[serde(rename = "aarch64-manylinux_2_34")]
    #[serde(alias = "aarch64-manylinux234")]
    Aarch64Manylinux234,

    /// An ARM64 target for the `manylinux_2_35` platform.
    #[cfg_attr(feature = "clap", value(name = "aarch64-manylinux_2_35"))]
    #[serde(rename = "aarch64-manylinux_2_35")]
    #[serde(alias = "aarch64-manylinux235")]
    Aarch64Manylinux235,

    /// An ARM64 target for the `manylinux_2_36` platform.
    #[cfg_attr(feature = "clap", value(name = "aarch64-manylinux_2_36"))]
    #[serde(rename = "aarch64-manylinux_2_36")]
    #[serde(alias = "aarch64-manylinux236")]
    Aarch64Manylinux236,

    /// An ARM64 target for the `manylinux_2_37` platform.
    #[cfg_attr(feature = "clap", value(name = "aarch64-manylinux_2_37"))]
    #[serde(rename = "aarch64-manylinux_2_37")]
    #[serde(alias = "aarch64-manylinux237")]
    Aarch64Manylinux237,

    /// An ARM64 target for the `manylinux_2_38` platform.
    #[cfg_attr(feature = "clap", value(name = "aarch64-manylinux_2_38"))]
    #[serde(rename = "aarch64-manylinux_2_38")]
    #[serde(alias = "aarch64-manylinux238")]
    Aarch64Manylinux238,

    /// An ARM64 target for the `manylinux_2_39` platform.
    #[cfg_attr(feature = "clap", value(name = "aarch64-manylinux_2_39"))]
    #[serde(rename = "aarch64-manylinux_2_39")]
    #[serde(alias = "aarch64-manylinux239")]
    Aarch64Manylinux239,

    /// An ARM64 target for the `manylinux_2_40` platform.
    #[cfg_attr(feature = "clap", value(name = "aarch64-manylinux_2_40"))]
    #[serde(rename = "aarch64-manylinux_2_40")]
    #[serde(alias = "aarch64-manylinux240")]
    Aarch64Manylinux240,

    /// A wasm32 target using the Pyodide 2024 platform. Meant for use with Python 3.12.
    #[cfg_attr(feature = "clap", value(name = "wasm32-pyodide2024"))]
    Wasm32Pyodide2024,
}

impl TargetTriple {
    /// Return the [`Platform`] for the target.
    pub fn platform(self) -> Platform {
        match self {
            Self::Windows | Self::X8664PcWindowsMsvc => Platform::new(Os::Windows, Arch::X86_64),
            Self::Linux | Self::X8664UnknownLinuxGnu => Platform::new(
                Os::Manylinux {
                    major: 2,
                    minor: 28,
                },
                Arch::X86_64,
            ),
            Self::Macos | Self::Aarch64AppleDarwin => {
                let (major, minor) = macos_deployment_target().map_or((13, 0), |(major, minor)| {
                    debug!("Found macOS deployment target: {}.{}", major, minor);
                    (major, minor)
                });
                Platform::new(Os::Macos { major, minor }, Arch::Aarch64)
            }
            Self::I686PcWindowsMsvc => Platform::new(Os::Windows, Arch::X86),
            Self::X8664AppleDarwin => {
                let (major, minor) = macos_deployment_target().map_or((13, 0), |(major, minor)| {
                    debug!("Found macOS deployment target: {}.{}", major, minor);
                    (major, minor)
                });
                Platform::new(Os::Macos { major, minor }, Arch::X86_64)
            }
            Self::Aarch64UnknownLinuxGnu => Platform::new(
                Os::Manylinux {
                    major: 2,
                    minor: 28,
                },
                Arch::Aarch64,
            ),
            Self::Aarch64UnknownLinuxMusl => {
                Platform::new(Os::Musllinux { major: 1, minor: 2 }, Arch::Aarch64)
            }
            Self::X8664UnknownLinuxMusl => {
                Platform::new(Os::Musllinux { major: 1, minor: 2 }, Arch::X86_64)
            }
            Self::X8664Manylinux2014 => Platform::new(
                Os::Manylinux {
                    major: 2,
                    minor: 17,
                },
                Arch::X86_64,
            ),
            Self::X8664Manylinux217 => Platform::new(
                Os::Manylinux {
                    major: 2,
                    minor: 17,
                },
                Arch::X86_64,
            ),
            Self::X8664Manylinux228 => Platform::new(
                Os::Manylinux {
                    major: 2,
                    minor: 28,
                },
                Arch::X86_64,
            ),
            Self::X8664Manylinux231 => Platform::new(
                Os::Manylinux {
                    major: 2,
                    minor: 31,
                },
                Arch::X86_64,
            ),
            Self::X8664Manylinux232 => Platform::new(
                Os::Manylinux {
                    major: 2,
                    minor: 32,
                },
                Arch::X86_64,
            ),
            Self::X8664Manylinux233 => Platform::new(
                Os::Manylinux {
                    major: 2,
                    minor: 33,
                },
                Arch::X86_64,
            ),
            Self::X8664Manylinux234 => Platform::new(
                Os::Manylinux {
                    major: 2,
                    minor: 34,
                },
                Arch::X86_64,
            ),
            Self::X8664Manylinux235 => Platform::new(
                Os::Manylinux {
                    major: 2,
                    minor: 35,
                },
                Arch::X86_64,
            ),
            Self::X8664Manylinux236 => Platform::new(
                Os::Manylinux {
                    major: 2,
                    minor: 36,
                },
                Arch::X86_64,
            ),
            Self::X8664Manylinux237 => Platform::new(
                Os::Manylinux {
                    major: 2,
                    minor: 37,
                },
                Arch::X86_64,
            ),
            Self::X8664Manylinux238 => Platform::new(
                Os::Manylinux {
                    major: 2,
                    minor: 38,
                },
                Arch::X86_64,
            ),
            Self::X8664Manylinux239 => Platform::new(
                Os::Manylinux {
                    major: 2,
                    minor: 39,
                },
                Arch::X86_64,
            ),
            Self::X8664Manylinux240 => Platform::new(
                Os::Manylinux {
                    major: 2,
                    minor: 40,
                },
                Arch::X86_64,
            ),
            Self::Aarch64Manylinux2014 => Platform::new(
                Os::Manylinux {
                    major: 2,
                    minor: 17,
                },
                Arch::Aarch64,
            ),
            Self::Aarch64Manylinux217 => Platform::new(
                Os::Manylinux {
                    major: 2,
                    minor: 17,
                },
                Arch::Aarch64,
            ),
            Self::Aarch64Manylinux228 => Platform::new(
                Os::Manylinux {
                    major: 2,
                    minor: 28,
                },
                Arch::Aarch64,
            ),
            Self::Aarch64Manylinux231 => Platform::new(
                Os::Manylinux {
                    major: 2,
                    minor: 31,
                },
                Arch::Aarch64,
            ),
            Self::Aarch64Manylinux232 => Platform::new(
                Os::Manylinux {
                    major: 2,
                    minor: 32,
                },
                Arch::Aarch64,
            ),
            Self::Aarch64Manylinux233 => Platform::new(
                Os::Manylinux {
                    major: 2,
                    minor: 33,
                },
                Arch::Aarch64,
            ),
            Self::Aarch64Manylinux234 => Platform::new(
                Os::Manylinux {
                    major: 2,
                    minor: 34,
                },
                Arch::Aarch64,
            ),
            Self::Aarch64Manylinux235 => Platform::new(
                Os::Manylinux {
                    major: 2,
                    minor: 35,
                },
                Arch::Aarch64,
            ),
            Self::Aarch64Manylinux236 => Platform::new(
                Os::Manylinux {
                    major: 2,
                    minor: 36,
                },
                Arch::Aarch64,
            ),
            Self::Aarch64Manylinux237 => Platform::new(
                Os::Manylinux {
                    major: 2,
                    minor: 37,
                },
                Arch::Aarch64,
            ),
            Self::Aarch64Manylinux238 => Platform::new(
                Os::Manylinux {
                    major: 2,
                    minor: 38,
                },
                Arch::Aarch64,
            ),
            Self::Aarch64Manylinux239 => Platform::new(
                Os::Manylinux {
                    major: 2,
                    minor: 39,
                },
                Arch::Aarch64,
            ),
            Self::Aarch64Manylinux240 => Platform::new(
                Os::Manylinux {
                    major: 2,
                    minor: 40,
                },
                Arch::Aarch64,
            ),
            Self::Wasm32Pyodide2024 => Platform::new(
                Os::Pyodide {
                    major: 2024,
                    minor: 0,
                },
                Arch::Wasm32,
            ),
        }
    }

    /// Return the `platform_machine` value for the target.
    pub fn platform_machine(self) -> &'static str {
        match self {
            Self::Windows | Self::X8664PcWindowsMsvc => "x86_64",
            Self::Linux | Self::X8664UnknownLinuxGnu => "x86_64",
            Self::Macos | Self::Aarch64AppleDarwin => "arm64",
            Self::I686PcWindowsMsvc => "x86",
            Self::X8664AppleDarwin => "x86_64",
            Self::Aarch64UnknownLinuxGnu => "aarch64",
            Self::Aarch64UnknownLinuxMusl => "aarch64",
            Self::X8664UnknownLinuxMusl => "x86_64",
            Self::X8664Manylinux2014 => "x86_64",
            Self::X8664Manylinux217 => "x86_64",
            Self::X8664Manylinux228 => "x86_64",
            Self::X8664Manylinux231 => "x86_64",
            Self::X8664Manylinux232 => "x86_64",
            Self::X8664Manylinux233 => "x86_64",
            Self::X8664Manylinux234 => "x86_64",
            Self::X8664Manylinux235 => "x86_64",
            Self::X8664Manylinux236 => "x86_64",
            Self::X8664Manylinux237 => "x86_64",
            Self::X8664Manylinux238 => "x86_64",
            Self::X8664Manylinux239 => "x86_64",
            Self::X8664Manylinux240 => "x86_64",
            Self::Aarch64Manylinux2014 => "aarch64",
            Self::Aarch64Manylinux217 => "aarch64",
            Self::Aarch64Manylinux228 => "aarch64",
            Self::Aarch64Manylinux231 => "aarch64",
            Self::Aarch64Manylinux232 => "aarch64",
            Self::Aarch64Manylinux233 => "aarch64",
            Self::Aarch64Manylinux234 => "aarch64",
            Self::Aarch64Manylinux235 => "aarch64",
            Self::Aarch64Manylinux236 => "aarch64",
            Self::Aarch64Manylinux237 => "aarch64",
            Self::Aarch64Manylinux238 => "aarch64",
            Self::Aarch64Manylinux239 => "aarch64",
            Self::Aarch64Manylinux240 => "aarch64",
            Self::Wasm32Pyodide2024 => "wasm32",
        }
    }

    /// Return the `platform_system` value for the target.
    pub fn platform_system(self) -> &'static str {
        match self {
            Self::Windows | Self::X8664PcWindowsMsvc => "Windows",
            Self::Linux | Self::X8664UnknownLinuxGnu => "Linux",
            Self::Macos | Self::Aarch64AppleDarwin => "Darwin",
            Self::I686PcWindowsMsvc => "Windows",
            Self::X8664AppleDarwin => "Darwin",
            Self::Aarch64UnknownLinuxGnu => "Linux",
            Self::Aarch64UnknownLinuxMusl => "Linux",
            Self::X8664UnknownLinuxMusl => "Linux",
            Self::X8664Manylinux2014 => "Linux",
            Self::X8664Manylinux217 => "Linux",
            Self::X8664Manylinux228 => "Linux",
            Self::X8664Manylinux231 => "Linux",
            Self::X8664Manylinux232 => "Linux",
            Self::X8664Manylinux233 => "Linux",
            Self::X8664Manylinux234 => "Linux",
            Self::X8664Manylinux235 => "Linux",
            Self::X8664Manylinux236 => "Linux",
            Self::X8664Manylinux237 => "Linux",
            Self::X8664Manylinux238 => "Linux",
            Self::X8664Manylinux239 => "Linux",
            Self::X8664Manylinux240 => "Linux",
            Self::Aarch64Manylinux2014 => "Linux",
            Self::Aarch64Manylinux217 => "Linux",
            Self::Aarch64Manylinux228 => "Linux",
            Self::Aarch64Manylinux231 => "Linux",
            Self::Aarch64Manylinux232 => "Linux",
            Self::Aarch64Manylinux233 => "Linux",
            Self::Aarch64Manylinux234 => "Linux",
            Self::Aarch64Manylinux235 => "Linux",
            Self::Aarch64Manylinux236 => "Linux",
            Self::Aarch64Manylinux237 => "Linux",
            Self::Aarch64Manylinux238 => "Linux",
            Self::Aarch64Manylinux239 => "Linux",
            Self::Aarch64Manylinux240 => "Linux",
            Self::Wasm32Pyodide2024 => "Emscripten",
        }
    }

    /// Return the `platform_version` value for the target.
    pub fn platform_version(self) -> &'static str {
        match self {
            Self::Windows | Self::X8664PcWindowsMsvc => "",
            Self::Linux | Self::X8664UnknownLinuxGnu => "",
            Self::Macos | Self::Aarch64AppleDarwin => "",
            Self::I686PcWindowsMsvc => "",
            Self::X8664AppleDarwin => "",
            Self::Aarch64UnknownLinuxGnu => "",
            Self::Aarch64UnknownLinuxMusl => "",
            Self::X8664UnknownLinuxMusl => "",
            Self::X8664Manylinux2014 => "",
            Self::X8664Manylinux217 => "",
            Self::X8664Manylinux228 => "",
            Self::X8664Manylinux231 => "",
            Self::X8664Manylinux232 => "",
            Self::X8664Manylinux233 => "",
            Self::X8664Manylinux234 => "",
            Self::X8664Manylinux235 => "",
            Self::X8664Manylinux236 => "",
            Self::X8664Manylinux237 => "",
            Self::X8664Manylinux238 => "",
            Self::X8664Manylinux239 => "",
            Self::X8664Manylinux240 => "",
            Self::Aarch64Manylinux2014 => "",
            Self::Aarch64Manylinux217 => "",
            Self::Aarch64Manylinux228 => "",
            Self::Aarch64Manylinux231 => "",
            Self::Aarch64Manylinux232 => "",
            Self::Aarch64Manylinux233 => "",
            Self::Aarch64Manylinux234 => "",
            Self::Aarch64Manylinux235 => "",
            Self::Aarch64Manylinux236 => "",
            Self::Aarch64Manylinux237 => "",
            Self::Aarch64Manylinux238 => "",
            Self::Aarch64Manylinux239 => "",
            Self::Aarch64Manylinux240 => "",
            // This is the value Emscripten gives for its version:
            // https://github.com/emscripten-core/emscripten/blob/4.0.8/system/lib/libc/emscripten_syscall_stubs.c#L63
            // It doesn't really seem to mean anything? But for completeness we include it here.
            Self::Wasm32Pyodide2024 => "#1",
        }
    }

    /// Return the `platform_release` value for the target.
    pub fn platform_release(self) -> &'static str {
        match self {
            Self::Windows | Self::X8664PcWindowsMsvc => "",
            Self::Linux | Self::X8664UnknownLinuxGnu => "",
            Self::Macos | Self::Aarch64AppleDarwin => "",
            Self::I686PcWindowsMsvc => "",
            Self::X8664AppleDarwin => "",
            Self::Aarch64UnknownLinuxGnu => "",
            Self::Aarch64UnknownLinuxMusl => "",
            Self::X8664UnknownLinuxMusl => "",
            Self::X8664Manylinux2014 => "",
            Self::X8664Manylinux217 => "",
            Self::X8664Manylinux228 => "",
            Self::X8664Manylinux231 => "",
            Self::X8664Manylinux232 => "",
            Self::X8664Manylinux233 => "",
            Self::X8664Manylinux234 => "",
            Self::X8664Manylinux235 => "",
            Self::X8664Manylinux236 => "",
            Self::X8664Manylinux237 => "",
            Self::X8664Manylinux238 => "",
            Self::X8664Manylinux239 => "",
            Self::X8664Manylinux240 => "",
            Self::Aarch64Manylinux2014 => "",
            Self::Aarch64Manylinux217 => "",
            Self::Aarch64Manylinux228 => "",
            Self::Aarch64Manylinux231 => "",
            Self::Aarch64Manylinux232 => "",
            Self::Aarch64Manylinux233 => "",
            Self::Aarch64Manylinux234 => "",
            Self::Aarch64Manylinux235 => "",
            Self::Aarch64Manylinux236 => "",
            Self::Aarch64Manylinux237 => "",
            Self::Aarch64Manylinux238 => "",
            Self::Aarch64Manylinux239 => "",
            Self::Aarch64Manylinux240 => "",
            // This is the Emscripten compiler version for Pyodide 2024.
            // See https://pyodide.org/en/stable/development/abi.html#pyodide-2024-0
            Self::Wasm32Pyodide2024 => "3.1.58",
        }
    }

    /// Return the `os_name` value for the target.
    pub fn os_name(self) -> &'static str {
        match self {
            Self::Windows | Self::X8664PcWindowsMsvc => "nt",
            Self::Linux | Self::X8664UnknownLinuxGnu => "posix",
            Self::Macos | Self::Aarch64AppleDarwin => "posix",
            Self::I686PcWindowsMsvc => "nt",
            Self::X8664AppleDarwin => "posix",
            Self::Aarch64UnknownLinuxGnu => "posix",
            Self::Aarch64UnknownLinuxMusl => "posix",
            Self::X8664UnknownLinuxMusl => "posix",
            Self::X8664Manylinux2014 => "posix",
            Self::X8664Manylinux217 => "posix",
            Self::X8664Manylinux228 => "posix",
            Self::X8664Manylinux231 => "posix",
            Self::X8664Manylinux232 => "posix",
            Self::X8664Manylinux233 => "posix",
            Self::X8664Manylinux234 => "posix",
            Self::X8664Manylinux235 => "posix",
            Self::X8664Manylinux236 => "posix",
            Self::X8664Manylinux237 => "posix",
            Self::X8664Manylinux238 => "posix",
            Self::X8664Manylinux239 => "posix",
            Self::X8664Manylinux240 => "posix",
            Self::Aarch64Manylinux2014 => "posix",
            Self::Aarch64Manylinux217 => "posix",
            Self::Aarch64Manylinux228 => "posix",
            Self::Aarch64Manylinux231 => "posix",
            Self::Aarch64Manylinux232 => "posix",
            Self::Aarch64Manylinux233 => "posix",
            Self::Aarch64Manylinux234 => "posix",
            Self::Aarch64Manylinux235 => "posix",
            Self::Aarch64Manylinux236 => "posix",
            Self::Aarch64Manylinux237 => "posix",
            Self::Aarch64Manylinux238 => "posix",
            Self::Aarch64Manylinux239 => "posix",
            Self::Aarch64Manylinux240 => "posix",
            Self::Wasm32Pyodide2024 => "posix",
        }
    }

    /// Return the `sys_platform` value for the target.
    pub fn sys_platform(self) -> &'static str {
        match self {
            Self::Windows | Self::X8664PcWindowsMsvc => "win32",
            Self::Linux | Self::X8664UnknownLinuxGnu => "linux",
            Self::Macos | Self::Aarch64AppleDarwin => "darwin",
            Self::I686PcWindowsMsvc => "win32",
            Self::X8664AppleDarwin => "darwin",
            Self::Aarch64UnknownLinuxGnu => "linux",
            Self::Aarch64UnknownLinuxMusl => "linux",
            Self::X8664UnknownLinuxMusl => "linux",
            Self::X8664Manylinux2014 => "linux",
            Self::X8664Manylinux217 => "linux",
            Self::X8664Manylinux228 => "linux",
            Self::X8664Manylinux231 => "linux",
            Self::X8664Manylinux232 => "linux",
            Self::X8664Manylinux233 => "linux",
            Self::X8664Manylinux234 => "linux",
            Self::X8664Manylinux235 => "linux",
            Self::X8664Manylinux236 => "linux",
            Self::X8664Manylinux237 => "linux",
            Self::X8664Manylinux238 => "linux",
            Self::X8664Manylinux239 => "linux",
            Self::X8664Manylinux240 => "linux",
            Self::Aarch64Manylinux2014 => "linux",
            Self::Aarch64Manylinux217 => "linux",
            Self::Aarch64Manylinux228 => "linux",
            Self::Aarch64Manylinux231 => "linux",
            Self::Aarch64Manylinux232 => "linux",
            Self::Aarch64Manylinux233 => "linux",
            Self::Aarch64Manylinux234 => "linux",
            Self::Aarch64Manylinux235 => "linux",
            Self::Aarch64Manylinux236 => "linux",
            Self::Aarch64Manylinux237 => "linux",
            Self::Aarch64Manylinux238 => "linux",
            Self::Aarch64Manylinux239 => "linux",
            Self::Aarch64Manylinux240 => "linux",
            Self::Wasm32Pyodide2024 => "emscripten",
        }
    }

    /// Return `true` if the platform is compatible with manylinux.
    pub fn manylinux_compatible(self) -> bool {
        match self {
            Self::Windows | Self::X8664PcWindowsMsvc => false,
            Self::Linux | Self::X8664UnknownLinuxGnu => true,
            Self::Macos | Self::Aarch64AppleDarwin => false,
            Self::I686PcWindowsMsvc => false,
            Self::X8664AppleDarwin => false,
            Self::Aarch64UnknownLinuxGnu => true,
            Self::Aarch64UnknownLinuxMusl => true,
            Self::X8664UnknownLinuxMusl => true,
            Self::X8664Manylinux2014 => true,
            Self::X8664Manylinux217 => true,
            Self::X8664Manylinux228 => true,
            Self::X8664Manylinux231 => true,
            Self::X8664Manylinux232 => true,
            Self::X8664Manylinux233 => true,
            Self::X8664Manylinux234 => true,
            Self::X8664Manylinux235 => true,
            Self::X8664Manylinux236 => true,
            Self::X8664Manylinux237 => true,
            Self::X8664Manylinux238 => true,
            Self::X8664Manylinux239 => true,
            Self::X8664Manylinux240 => true,
            Self::Aarch64Manylinux2014 => true,
            Self::Aarch64Manylinux217 => true,
            Self::Aarch64Manylinux228 => true,
            Self::Aarch64Manylinux231 => true,
            Self::Aarch64Manylinux232 => true,
            Self::Aarch64Manylinux233 => true,
            Self::Aarch64Manylinux234 => true,
            Self::Aarch64Manylinux235 => true,
            Self::Aarch64Manylinux236 => true,
            Self::Aarch64Manylinux237 => true,
            Self::Aarch64Manylinux238 => true,
            Self::Aarch64Manylinux239 => true,
            Self::Aarch64Manylinux240 => true,
            Self::Wasm32Pyodide2024 => false,
        }
    }

    /// Return a [`MarkerEnvironment`] compatible with the given [`TargetTriple`], based on
    /// a base [`MarkerEnvironment`].
    ///
    /// The returned [`MarkerEnvironment`] will preserve the base environment's Python version
    /// markers, but override its platform markers.
    pub fn markers(self, base: &MarkerEnvironment) -> MarkerEnvironment {
        base.clone()
            .with_os_name(self.os_name())
            .with_platform_machine(self.platform_machine())
            .with_platform_system(self.platform_system())
            .with_sys_platform(self.sys_platform())
            .with_platform_release(self.platform_release())
            .with_platform_version(self.platform_version())
    }
}

/// Return the macOS deployment target as parsed from the environment.
fn macos_deployment_target() -> Option<(u16, u16)> {
    let version = std::env::var(EnvVars::MACOSX_DEPLOYMENT_TARGET).ok()?;
    let mut parts = version.split('.');

    // Parse the major version (e.g., `12` in `12.0`).
    let major = parts.next()?.parse::<u16>().ok()?;

    // Parse the minor version (e.g., `0` in `12.0`), with a default of `0`.
    let minor = parts.next().unwrap_or("0").parse::<u16>().ok()?;

    Some((major, minor))
}

# Generated by rust2rpm 22
%bcond_without check
%global debug_package %{nil}

%global crate litrs

Name:           rust-litrs
Version:        0.2.3
Release:        %autorelease
Summary:        Parse and inspect Rust literals (i.e

# Upstream license specification: MIT/Apache-2.0
License:        MIT OR Apache-2.0
URL:            https://crates.io/crates/litrs
Source:         %{crates_source}

ExclusiveArch:  %{rust_arches}

BuildRequires:  rust-packaging >= 21

%global _description %{expand:
Parse and inspect Rust literals (i.e. tokens in the Rust programming language
representing fixed values). Particularly useful for proc macros, but can also
be used outside of a proc-macro context.}

%description %{_description}

%package        devel
Summary:        %{summary}
BuildArch:      noarch

%description    devel %{_description}

This package contains library source intended for building other packages which
use the "%{crate}" crate.

%files          devel
%license %{crate_instdir}/LICENSE-APACHE
%license %{crate_instdir}/LICENSE-MIT
%doc %{crate_instdir}/CHANGELOG.md
%doc %{crate_instdir}/README.md
%{crate_instdir}/

%package     -n %{name}+default-devel
Summary:        %{summary}
BuildArch:      noarch

%description -n %{name}+default-devel %{_description}

This package contains library source intended for building other packages which
use the "default" feature of the "%{crate}" crate.

%files       -n %{name}+default-devel
%ghost %{crate_instdir}/Cargo.toml

%package     -n %{name}+proc-macro2-devel
Summary:        %{summary}
BuildArch:      noarch

%description -n %{name}+proc-macro2-devel %{_description}

This package contains library source intended for building other packages which
use the "proc-macro2" feature of the "%{crate}" crate.

%files       -n %{name}+proc-macro2-devel
%ghost %{crate_instdir}/Cargo.toml

%prep
%autosetup -n %{crate}-%{version_no_tilde} -p1
%cargo_prep

%generate_buildrequires
%cargo_generate_buildrequires

%build
%cargo_build

%install
%cargo_install

%if %{with check}
%check
%cargo_test
%endif

%changelog
%autochangelog
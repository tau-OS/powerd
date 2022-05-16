%bcond_without check

%global crate powerd

Name:           %{crate}
Version:        1.0.0
Release:        %autorelease
Summary:        Power management daemon that provides power management featues and optimizations

# Upstream license specification: GPL-3.0-only
License:        GPLv3
URL:            https://tauos.co
Source:         %{crates_source}

ExclusiveArch:  %{rust_arches}

BuildRequires:  rust-packaging >= 21

%global _description %{expand:
Power management daemon that provides power management featues and
optimizations.}

%description %{_description}

%package     -n %{crate}
Summary:        %{summary}

%description -n %{crate} %{_description}

%files       -n %{crate}
%license LICENSE
%{_bindir}/powerd

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
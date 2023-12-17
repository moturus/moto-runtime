// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[doc(hidden)]
pub extern crate core as _core;

#[macro_export(local_inner_macros)]
macro_rules! bitflags {
    (
        $(#[$outer:meta])*
        $vis:vis struct $BitFlags:ident: $T:ty {
            $(
                $(#[$inner:ident $($args:tt)*])*
                const $Flag:ident = $value:expr;
            )*
        }

        $($t:tt)*
    ) => {
        $(#[$outer])*
        #[derive(Copy, PartialEq, Eq, Clone, PartialOrd, Ord, Hash)]
        $vis struct $BitFlags {
            bits: $T,
        }

        __impl_bitflags! {
            $BitFlags: $T {
                $(
                    $(#[$inner $($args)*])*
                    $Flag = $value;
                )*
            }
        }

        bitflags! {
            $($t)*
        }
    };
    () => {};
}

pub use bitflags;

// A helper macro to implement the `all` function.
#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! __impl_all_bitflags {
    (
        $BitFlags:ident: $T:ty {
            $(
                $(#[$attr:ident $($args:tt)*])*
                $Flag:ident = $value:expr;
            )+
        }
    ) => {
        // See `Debug::fmt` for why this approach is taken.
        #[allow(non_snake_case)]
        trait __BitFlags {
            $(
                const $Flag: $T = 0;
            )+
        }
        #[allow(non_snake_case)]
        impl __BitFlags for $BitFlags {
            $(
                __impl_bitflags! {
                    #[allow(deprecated)]
                    $(? #[$attr $($args)*])*
                    const $Flag: $T = Self::$Flag.bits;
                }
            )+
        }
        Self { bits: $(<Self as __BitFlags>::$Flag)|+ }
    };
    (
        $BitFlags:ident: $T:ty { }
    ) => {
        Self { bits: 0 }
    };
}

#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! __impl_bitflags {
    (
        $BitFlags:ident: $T:ty {
            $(
                $(#[$attr:ident $($args:tt)*])*
                $Flag:ident = $value:expr;
            )*
        }
    ) => {
        impl $crate::_core::fmt::Debug for $BitFlags {
            fn fmt(&self, f: &mut $crate::_core::fmt::Formatter) -> $crate::_core::fmt::Result {
                // This convoluted approach is to handle #[cfg]-based flag
                // omission correctly. For example it needs to support:
                //
                //    #[cfg(unix)] const A: Flag = /* ... */;
                //    #[cfg(windows)] const B: Flag = /* ... */;

                // Unconditionally define a check for every flag, even disabled
                // ones.
                #[allow(non_snake_case)]
                trait __BitFlags {
                    $(
                        #[inline]
                        fn $Flag(&self) -> bool { false }
                    )*
                }

                // Conditionally override the check for just those flags that
                // are not #[cfg]ed away.
                #[allow(non_snake_case)]
                impl __BitFlags for $BitFlags {
                    $(
                        __impl_bitflags! {
                            #[allow(deprecated)]
                            #[inline]
                            $(? #[$attr $($args)*])*
                            fn $Flag(&self) -> bool {
                                if Self::$Flag.bits == 0 && self.bits != 0 {
                                    false
                                } else {
                                    self.bits & Self::$Flag.bits == Self::$Flag.bits
                                }
                            }
                        }
                    )*
                }

                let mut first = true;
                $(
                    if <Self as __BitFlags>::$Flag(self) {
                        if !first {
                            f.write_str(" | ")?;
                        }
                        first = false;
                        f.write_str($crate::_core::stringify!($Flag))?;
                    }
                )*
                let extra_bits = self.bits & !Self::all().bits();
                if extra_bits != 0 {
                    if !first {
                        f.write_str(" | ")?;
                    }
                    first = false;
                    f.write_str("0x")?;
                    $crate::_core::fmt::LowerHex::fmt(&extra_bits, f)?;
                }
                if first {
                    f.write_str("(empty)")?;
                }
                Ok(())
            }
        }
        impl $crate::_core::fmt::Binary for $BitFlags {
            fn fmt(&self, f: &mut $crate::_core::fmt::Formatter) -> $crate::_core::fmt::Result {
                $crate::_core::fmt::Binary::fmt(&self.bits, f)
            }
        }
        impl $crate::_core::fmt::Octal for $BitFlags {
            fn fmt(&self, f: &mut $crate::_core::fmt::Formatter) -> $crate::_core::fmt::Result {
                $crate::_core::fmt::Octal::fmt(&self.bits, f)
            }
        }
        impl $crate::_core::fmt::LowerHex for $BitFlags {
            fn fmt(&self, f: &mut $crate::_core::fmt::Formatter) -> $crate::_core::fmt::Result {
                $crate::_core::fmt::LowerHex::fmt(&self.bits, f)
            }
        }
        impl $crate::_core::fmt::UpperHex for $BitFlags {
            fn fmt(&self, f: &mut $crate::_core::fmt::Formatter) -> $crate::_core::fmt::Result {
                $crate::_core::fmt::UpperHex::fmt(&self.bits, f)
            }
        }

        #[allow(dead_code)]
        impl $BitFlags {
            $(
                $(#[$attr $($args)*])*
                pub const $Flag: Self = Self { bits: $value };
            )*

            /// Returns an empty set of flags.
            #[inline]
            pub const fn empty() -> Self {
                Self { bits: 0 }
            }

            /// Returns the set containing all flags.
            #[inline]
            pub const fn all() -> Self {
                __impl_all_bitflags! {
                    $BitFlags: $T {
                        $(
                            $(#[$attr $($args)*])*
                            $Flag = $value;
                        )*
                    }
                }
            }

            /// Returns the raw value of the flags currently stored.
            #[inline]
            pub const fn bits(&self) -> $T {
                self.bits
            }

            /// Convert from underlying bit representation, unless that
            /// representation contains bits that do not correspond to a flag.
            #[inline]
            pub const fn from_bits(bits: $T) -> $crate::_core::option::Option<Self> {
                if (bits & !Self::all().bits()) == 0 {
                    $crate::_core::option::Option::Some(Self { bits })
                } else {
                    $crate::_core::option::Option::None
                }
            }

            /// Convert from underlying bit representation, dropping any bits
            /// that do not correspond to flags.
            #[inline]
            pub const fn from_bits_truncate(bits: $T) -> Self {
                Self { bits: bits & Self::all().bits }
            }

            /// Convert from underlying bit representation, preserving all
            /// bits (even those not corresponding to a defined flag).
            ///
            /// # Safety
            ///
            /// The caller of the `bitflags!` macro can chose to allow or
            /// disallow extra bits for their bitflags type.
            ///
            /// The caller of `from_bits_unchecked()` has to ensure that
            /// all bits correspond to a defined flag or that extra bits
            /// are valid for this bitflags type.
            #[inline]
            pub const unsafe fn from_bits_unchecked(bits: $T) -> Self {
                Self { bits }
            }

            /// Returns `true` if no flags are currently stored.
            #[inline]
            pub const fn is_empty(&self) -> bool {
                self.bits() == Self::empty().bits()
            }

            /// Returns `true` if all flags are currently set.
            #[inline]
            pub const fn is_all(&self) -> bool {
                Self::all().bits | self.bits == self.bits
            }

            /// Returns `true` if there are flags common to both `self` and `other`.
            #[inline]
            pub const fn intersects(&self, other: Self) -> bool {
                !(Self { bits: self.bits & other.bits}).is_empty()
            }

            /// Returns `true` if all of the flags in `other` are contained within `self`.
            #[inline]
            pub const fn contains(&self, other: Self) -> bool {
                (self.bits & other.bits) == other.bits
            }

            /// Inserts the specified flags in-place.
            #[inline]
            pub fn insert(&mut self, other: Self) {
                self.bits |= other.bits;
            }

            /// Removes the specified flags in-place.
            #[inline]
            pub fn remove(&mut self, other: Self) {
                self.bits &= !other.bits;
            }

            /// Toggles the specified flags in-place.
            #[inline]
            pub fn toggle(&mut self, other: Self) {
                self.bits ^= other.bits;
            }

            /// Inserts or removes the specified flags depending on the passed value.
            #[inline]
            pub fn set(&mut self, other: Self, value: bool) {
                if value {
                    self.insert(other);
                } else {
                    self.remove(other);
                }
            }

            /// Returns the intersection between the flags in `self` and
            /// `other`.
            ///
            /// Specifically, the returned set contains only the flags which are
            /// present in *both* `self` *and* `other`.
            ///
            /// This is equivalent to using the `&` operator (e.g.
            /// [`ops::BitAnd`]), as in `flags & other`.
            ///
            /// [`ops::BitAnd`]: https://doc.rust-lang.org/std/ops/trait.BitAnd.html
            #[inline]
            #[must_use]
            pub const fn intersection(self, other: Self) -> Self {
                Self { bits: self.bits & other.bits }
            }

            /// Returns the union of between the flags in `self` and `other`.
            ///
            /// Specifically, the returned set contains all flags which are
            /// present in *either* `self` *or* `other`, including any which are
            /// present in both (see [`Self::symmetric_difference`] if that
            /// is undesirable).
            ///
            /// This is equivalent to using the `|` operator (e.g.
            /// [`ops::BitOr`]), as in `flags | other`.
            ///
            /// [`ops::BitOr`]: https://doc.rust-lang.org/std/ops/trait.BitOr.html
            #[inline]
            #[must_use]
            pub const fn union(self, other: Self) -> Self {
                Self { bits: self.bits | other.bits }
            }

            /// Returns the difference between the flags in `self` and `other`.
            ///
            /// Specifically, the returned set contains all flags present in
            /// `self`, except for the ones present in `other`.
            ///
            /// It is also conceptually equivalent to the "bit-clear" operation:
            /// `flags & !other` (and this syntax is also supported).
            ///
            /// This is equivalent to using the `-` operator (e.g.
            /// [`ops::Sub`]), as in `flags - other`.
            ///
            /// [`ops::Sub`]: https://doc.rust-lang.org/std/ops/trait.Sub.html
            #[inline]
            #[must_use]
            pub const fn difference(self, other: Self) -> Self {
                Self { bits: self.bits & !other.bits }
            }

            /// Returns the [symmetric difference][sym-diff] between the flags
            /// in `self` and `other`.
            ///
            /// Specifically, the returned set contains the flags present which
            /// are present in `self` or `other`, but that are not present in
            /// both. Equivalently, it contains the flags present in *exactly
            /// one* of the sets `self` and `other`.
            ///
            /// This is equivalent to using the `^` operator (e.g.
            /// [`ops::BitXor`]), as in `flags ^ other`.
            ///
            /// [sym-diff]: https://en.wikipedia.org/wiki/Symmetric_difference
            /// [`ops::BitXor`]: https://doc.rust-lang.org/std/ops/trait.BitXor.html
            #[inline]
            #[must_use]
            pub const fn symmetric_difference(self, other: Self) -> Self {
                Self { bits: self.bits ^ other.bits }
            }

            /// Returns the complement of this set of flags.
            ///
            /// Specifically, the returned set contains all the flags which are
            /// not set in `self`, but which are allowed for this type.
            ///
            /// Alternatively, it can be thought of as the set difference
            /// between [`Self::all()`] and `self` (e.g. `Self::all() - self`)
            ///
            /// This is equivalent to using the `!` operator (e.g.
            /// [`ops::Not`]), as in `!flags`.
            ///
            /// [`Self::all()`]: Self::all
            /// [`ops::Not`]: https://doc.rust-lang.org/std/ops/trait.Not.html
            #[inline]
            #[must_use]
            pub const fn complement(self) -> Self {
                Self::from_bits_truncate(!self.bits)
            }

        }

        impl $crate::_core::ops::BitOr for $BitFlags {
            type Output = Self;

            /// Returns the union of the two sets of flags.
            #[inline]
            fn bitor(self, other: $BitFlags) -> Self {
                Self { bits: self.bits | other.bits }
            }
        }

        impl $crate::_core::ops::BitOrAssign for $BitFlags {
            /// Adds the set of flags.
            #[inline]
            fn bitor_assign(&mut self, other: Self) {
                self.bits |= other.bits;
            }
        }

        impl $crate::_core::ops::BitXor for $BitFlags {
            type Output = Self;

            /// Returns the left flags, but with all the right flags toggled.
            #[inline]
            fn bitxor(self, other: Self) -> Self {
                Self { bits: self.bits ^ other.bits }
            }
        }

        impl $crate::_core::ops::BitXorAssign for $BitFlags {
            /// Toggles the set of flags.
            #[inline]
            fn bitxor_assign(&mut self, other: Self) {
                self.bits ^= other.bits;
            }
        }

        impl $crate::_core::ops::BitAnd for $BitFlags {
            type Output = Self;

            /// Returns the intersection between the two sets of flags.
            #[inline]
            fn bitand(self, other: Self) -> Self {
                Self { bits: self.bits & other.bits }
            }
        }

        impl $crate::_core::ops::BitAndAssign for $BitFlags {
            /// Disables all flags disabled in the set.
            #[inline]
            fn bitand_assign(&mut self, other: Self) {
                self.bits &= other.bits;
            }
        }

        impl $crate::_core::ops::Sub for $BitFlags {
            type Output = Self;

            /// Returns the set difference of the two sets of flags.
            #[inline]
            fn sub(self, other: Self) -> Self {
                Self { bits: self.bits & !other.bits }
            }
        }

        impl $crate::_core::ops::SubAssign for $BitFlags {
            /// Disables all flags enabled in the set.
            #[inline]
            fn sub_assign(&mut self, other: Self) {
                self.bits &= !other.bits;
            }
        }

        impl $crate::_core::ops::Not for $BitFlags {
            type Output = Self;

            /// Returns the complement of this set of flags.
            #[inline]
            fn not(self) -> Self {
                Self { bits: !self.bits } & Self::all()
            }
        }

        impl $crate::_core::iter::Extend<$BitFlags> for $BitFlags {
            fn extend<T: $crate::_core::iter::IntoIterator<Item=Self>>(&mut self, iterator: T) {
                for item in iterator {
                    self.insert(item)
                }
            }
        }

        impl $crate::_core::iter::FromIterator<$BitFlags> for $BitFlags {
            fn from_iter<T: $crate::_core::iter::IntoIterator<Item=Self>>(iterator: T) -> Self {
                let mut result = Self::empty();
                result.extend(iterator);
                result
            }
        }
    };

    // Every attribute that the user writes on a const is applied to the
    // corresponding const that we generate, but within the implementation of
    // Debug and all() we want to ignore everything but #[cfg] attributes. In
    // particular, including a #[deprecated] attribute on those items would fail
    // to compile.
    // https://github.com/bitflags/bitflags/issues/109
    //
    // Input:
    //
    //     ? #[cfg(feature = "advanced")]
    //     ? #[deprecated(note = "Use something else.")]
    //     ? #[doc = r"High quality documentation."]
    //     fn f() -> i32 { /* ... */ }
    //
    // Output:
    //
    //     #[cfg(feature = "advanced")]
    //     fn f() -> i32 { /* ... */ }
    (
        $(#[$filtered:meta])*
        ? #[cfg $($cfgargs:tt)*]
        $(? #[$rest:ident $($restargs:tt)*])*
        fn $($item:tt)*
    ) => {
        __impl_bitflags! {
            $(#[$filtered])*
            #[cfg $($cfgargs)*]
            $(? #[$rest $($restargs)*])*
            fn $($item)*
        }
    };
    (
        $(#[$filtered:meta])*
        // $next != `cfg`
        ? #[$next:ident $($nextargs:tt)*]
        $(? #[$rest:ident $($restargs:tt)*])*
        fn $($item:tt)*
    ) => {
        __impl_bitflags! {
            $(#[$filtered])*
            // $next filtered out
            $(? #[$rest $($restargs)*])*
            fn $($item)*
        }
    };
    (
        $(#[$filtered:meta])*
        fn $($item:tt)*
    ) => {
        $(#[$filtered])*
        fn $($item)*
    };

    // Every attribute that the user writes on a const is applied to the
    // corresponding const that we generate, but within the implementation of
    // Debug and all() we want to ignore everything but #[cfg] attributes. In
    // particular, including a #[deprecated] attribute on those items would fail
    // to compile.
    // https://github.com/bitflags/bitflags/issues/109
    //
    // const version
    //
    // Input:
    //
    //     ? #[cfg(feature = "advanced")]
    //     ? #[deprecated(note = "Use something else.")]
    //     ? #[doc = r"High quality documentation."]
    //     const f: i32 { /* ... */ }
    //
    // Output:
    //
    //     #[cfg(feature = "advanced")]
    //     const f: i32 { /* ... */ }
    (
        $(#[$filtered:meta])*
        ? #[cfg $($cfgargs:tt)*]
        $(? #[$rest:ident $($restargs:tt)*])*
        const $($item:tt)*
    ) => {
        __impl_bitflags! {
            $(#[$filtered])*
            #[cfg $($cfgargs)*]
            $(? #[$rest $($restargs)*])*
            const $($item)*
        }
    };
    (
        $(#[$filtered:meta])*
        // $next != `cfg`
        ? #[$next:ident $($nextargs:tt)*]
        $(? #[$rest:ident $($restargs:tt)*])*
        const $($item:tt)*
    ) => {
        __impl_bitflags! {
            $(#[$filtered])*
            // $next filtered out
            $(? #[$rest $($restargs)*])*
            const $($item)*
        }
    };
    (
        $(#[$filtered:meta])*
        const $($item:tt)*
    ) => {
        $(#[$filtered])*
        const $($item)*
    };
}

#[cfg(feature = "example_generated")]
pub mod example_generated;

#[cfg(test)]
mod tests {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    bitflags! {
        #[doc = "> The first principle is that you must not fool yourself — and"]
        #[doc = "> you are the easiest person to fool."]
        #[doc = "> "]
        #[doc = "> - Richard Feynman"]
        #[derive(Default)]
        struct Flags: u32 {
            const A = 0b00000001;
            #[doc = "<pcwalton> macros are way better at generating code than trans is"]
            const B = 0b00000010;
            const C = 0b00000100;
            #[doc = "* cmr bed"]
            #[doc = "* strcat table"]
            #[doc = "<strcat> wait what?"]
            const ABC = Self::A.bits | Self::B.bits | Self::C.bits;
        }

        struct _CfgFlags: u32 {
            #[cfg(unix)]
            const _CFG_A = 0b01;
            #[cfg(windows)]
            const _CFG_B = 0b01;
            #[cfg(unix)]
            const _CFG_C = Self::_CFG_A.bits | 0b10;
        }

        struct AnotherSetOfFlags: i8 {
            const ANOTHER_FLAG = -1_i8;
        }

        struct LongFlags: u32 {
            const LONG_A = 0b1111111111111111;
        }
    }

    bitflags! {
        struct EmptyFlags: u32 {
        }
    }

    #[test]
    fn test_bits() {
        assert_eq!(Flags::empty().bits(), 0b00000000);
        assert_eq!(Flags::A.bits(), 0b00000001);
        assert_eq!(Flags::ABC.bits(), 0b00000111);

        assert_eq!(AnotherSetOfFlags::empty().bits(), 0b00);
        assert_eq!(AnotherSetOfFlags::ANOTHER_FLAG.bits(), !0_i8);

        assert_eq!(EmptyFlags::empty().bits(), 0b00000000);
    }

    #[test]
    fn test_from_bits() {
        assert_eq!(Flags::from_bits(0), Some(Flags::empty()));
        assert_eq!(Flags::from_bits(0b1), Some(Flags::A));
        assert_eq!(Flags::from_bits(0b10), Some(Flags::B));
        assert_eq!(Flags::from_bits(0b11), Some(Flags::A | Flags::B));
        assert_eq!(Flags::from_bits(0b1000), None);

        assert_eq!(
            AnotherSetOfFlags::from_bits(!0_i8),
            Some(AnotherSetOfFlags::ANOTHER_FLAG)
        );

        assert_eq!(EmptyFlags::from_bits(0), Some(EmptyFlags::empty()));
        assert_eq!(EmptyFlags::from_bits(0b1), None);
    }

    #[test]
    fn test_from_bits_truncate() {
        assert_eq!(Flags::from_bits_truncate(0), Flags::empty());
        assert_eq!(Flags::from_bits_truncate(0b1), Flags::A);
        assert_eq!(Flags::from_bits_truncate(0b10), Flags::B);
        assert_eq!(Flags::from_bits_truncate(0b11), (Flags::A | Flags::B));
        assert_eq!(Flags::from_bits_truncate(0b1000), Flags::empty());
        assert_eq!(Flags::from_bits_truncate(0b1001), Flags::A);

        assert_eq!(
            AnotherSetOfFlags::from_bits_truncate(0_i8),
            AnotherSetOfFlags::empty()
        );

        assert_eq!(EmptyFlags::from_bits_truncate(0), EmptyFlags::empty());
        assert_eq!(EmptyFlags::from_bits_truncate(0b1), EmptyFlags::empty());
    }

    #[test]
    fn test_from_bits_unchecked() {
        let extra = unsafe { Flags::from_bits_unchecked(0b1000) };
        assert_eq!(unsafe { Flags::from_bits_unchecked(0) }, Flags::empty());
        assert_eq!(unsafe { Flags::from_bits_unchecked(0b1) }, Flags::A);
        assert_eq!(unsafe { Flags::from_bits_unchecked(0b10) }, Flags::B);

        assert_eq!(
            unsafe { Flags::from_bits_unchecked(0b11) },
            (Flags::A | Flags::B)
        );
        assert_eq!(
            unsafe { Flags::from_bits_unchecked(0b1000) },
            (extra | Flags::empty())
        );
        assert_eq!(
            unsafe { Flags::from_bits_unchecked(0b1001) },
            (extra | Flags::A)
        );

        let extra = unsafe { EmptyFlags::from_bits_unchecked(0b1000) };
        assert_eq!(
            unsafe { EmptyFlags::from_bits_unchecked(0b1000) },
            (extra | EmptyFlags::empty())
        );
    }

    #[test]
    fn test_is_empty() {
        assert!(Flags::empty().is_empty());
        assert!(!Flags::A.is_empty());
        assert!(!Flags::ABC.is_empty());

        assert!(!AnotherSetOfFlags::ANOTHER_FLAG.is_empty());

        assert!(EmptyFlags::empty().is_empty());
        assert!(EmptyFlags::all().is_empty());
    }

    #[test]
    fn test_is_all() {
        assert!(Flags::all().is_all());
        assert!(!Flags::A.is_all());
        assert!(Flags::ABC.is_all());

        let extra = unsafe { Flags::from_bits_unchecked(0b1000) };
        assert!(!extra.is_all());
        assert!(!(Flags::A | extra).is_all());
        assert!((Flags::ABC | extra).is_all());

        assert!(AnotherSetOfFlags::ANOTHER_FLAG.is_all());

        assert!(EmptyFlags::all().is_all());
        assert!(EmptyFlags::empty().is_all());
    }

    #[test]
    fn test_two_empties_do_not_intersect() {
        let e1 = Flags::empty();
        let e2 = Flags::empty();
        assert!(!e1.intersects(e2));

        assert!(AnotherSetOfFlags::ANOTHER_FLAG.intersects(AnotherSetOfFlags::ANOTHER_FLAG));
    }

    #[test]
    fn test_empty_does_not_intersect_with_full() {
        let e1 = Flags::empty();
        let e2 = Flags::ABC;
        assert!(!e1.intersects(e2));
    }

    #[test]
    fn test_disjoint_intersects() {
        let e1 = Flags::A;
        let e2 = Flags::B;
        assert!(!e1.intersects(e2));
    }

    #[test]
    fn test_overlapping_intersects() {
        let e1 = Flags::A;
        let e2 = Flags::A | Flags::B;
        assert!(e1.intersects(e2));
    }

    #[test]
    fn test_contains() {
        let e1 = Flags::A;
        let e2 = Flags::A | Flags::B;
        assert!(!e1.contains(e2));
        assert!(e2.contains(e1));
        assert!(Flags::ABC.contains(e2));

        assert!(AnotherSetOfFlags::ANOTHER_FLAG.contains(AnotherSetOfFlags::ANOTHER_FLAG));

        assert!(EmptyFlags::empty().contains(EmptyFlags::empty()));
    }

    #[test]
    fn test_insert() {
        let mut e1 = Flags::A;
        let e2 = Flags::A | Flags::B;
        e1.insert(e2);
        assert_eq!(e1, e2);

        let mut e3 = AnotherSetOfFlags::empty();
        e3.insert(AnotherSetOfFlags::ANOTHER_FLAG);
        assert_eq!(e3, AnotherSetOfFlags::ANOTHER_FLAG);
    }

    #[test]
    fn test_remove() {
        let mut e1 = Flags::A | Flags::B;
        let e2 = Flags::A | Flags::C;
        e1.remove(e2);
        assert_eq!(e1, Flags::B);

        let mut e3 = AnotherSetOfFlags::ANOTHER_FLAG;
        e3.remove(AnotherSetOfFlags::ANOTHER_FLAG);
        assert_eq!(e3, AnotherSetOfFlags::empty());
    }

    #[test]
    fn test_operators() {
        let e1 = Flags::A | Flags::C;
        let e2 = Flags::B | Flags::C;
        assert_eq!((e1 | e2), Flags::ABC); // union
        assert_eq!((e1 & e2), Flags::C); // intersection
        assert_eq!((e1 - e2), Flags::A); // set difference
        assert_eq!(!e2, Flags::A); // set complement
        assert_eq!(e1 ^ e2, Flags::A | Flags::B); // toggle
        let mut e3 = e1;
        e3.toggle(e2);
        assert_eq!(e3, Flags::A | Flags::B);

        let mut m4 = AnotherSetOfFlags::empty();
        m4.toggle(AnotherSetOfFlags::empty());
        assert_eq!(m4, AnotherSetOfFlags::empty());
    }

    #[test]
    fn test_operators_unchecked() {
        let extra = unsafe { Flags::from_bits_unchecked(0b1000) };
        let e1 = Flags::A | Flags::C | extra;
        let e2 = Flags::B | Flags::C;
        assert_eq!((e1 | e2), (Flags::ABC | extra)); // union
        assert_eq!((e1 & e2), Flags::C); // intersection
        assert_eq!((e1 - e2), (Flags::A | extra)); // set difference
        assert_eq!(!e2, Flags::A); // set complement
        assert_eq!(!e1, Flags::B); // set complement
        assert_eq!(e1 ^ e2, Flags::A | Flags::B | extra); // toggle
        let mut e3 = e1;
        e3.toggle(e2);
        assert_eq!(e3, Flags::A | Flags::B | extra);
    }

    #[test]
    fn test_set_ops_basic() {
        let ab = Flags::A.union(Flags::B);
        let ac = Flags::A.union(Flags::C);
        let bc = Flags::B.union(Flags::C);
        assert_eq!(ab.bits, 0b011);
        assert_eq!(bc.bits, 0b110);
        assert_eq!(ac.bits, 0b101);

        assert_eq!(ab, Flags::B.union(Flags::A));
        assert_eq!(ac, Flags::C.union(Flags::A));
        assert_eq!(bc, Flags::C.union(Flags::B));

        assert_eq!(ac, Flags::A | Flags::C);
        assert_eq!(bc, Flags::B | Flags::C);
        assert_eq!(ab.union(bc), Flags::ABC);

        assert_eq!(ac, Flags::A | Flags::C);
        assert_eq!(bc, Flags::B | Flags::C);

        assert_eq!(ac.union(bc), ac | bc);
        assert_eq!(ac.union(bc), Flags::ABC);
        assert_eq!(bc.union(ac), Flags::ABC);

        assert_eq!(ac.intersection(bc), ac & bc);
        assert_eq!(ac.intersection(bc), Flags::C);
        assert_eq!(bc.intersection(ac), Flags::C);

        assert_eq!(ac.difference(bc), ac - bc);
        assert_eq!(bc.difference(ac), bc - ac);
        assert_eq!(ac.difference(bc), Flags::A);
        assert_eq!(bc.difference(ac), Flags::B);

        assert_eq!(bc.complement(), !bc);
        assert_eq!(bc.complement(), Flags::A);
        assert_eq!(ac.symmetric_difference(bc), Flags::A.union(Flags::B));
        assert_eq!(bc.symmetric_difference(ac), Flags::A.union(Flags::B));
    }

    #[test]
    fn test_set_ops_const() {
        // These just test that these compile and don't cause use-site panics
        // (would be possible if we had some sort of UB)
        const INTERSECT: Flags = Flags::all().intersection(Flags::C);
        const UNION: Flags = Flags::A.union(Flags::C);
        const DIFFERENCE: Flags = Flags::all().difference(Flags::A);
        const COMPLEMENT: Flags = Flags::C.complement();
        const SYM_DIFFERENCE: Flags = UNION.symmetric_difference(DIFFERENCE);
        assert_eq!(INTERSECT, Flags::C);
        assert_eq!(UNION, Flags::A | Flags::C);
        assert_eq!(DIFFERENCE, Flags::all() - Flags::A);
        assert_eq!(COMPLEMENT, !Flags::C);
        assert_eq!(
            SYM_DIFFERENCE,
            (Flags::A | Flags::C) ^ (Flags::all() - Flags::A)
        );
    }

    #[test]
    fn test_set_ops_unchecked() {
        let extra = unsafe { Flags::from_bits_unchecked(0b1000) };
        let e1 = Flags::A.union(Flags::C).union(extra);
        let e2 = Flags::B.union(Flags::C);
        assert_eq!(e1.bits, 0b1101);
        assert_eq!(e1.union(e2), (Flags::ABC | extra));
        assert_eq!(e1.intersection(e2), Flags::C);
        assert_eq!(e1.difference(e2), Flags::A | extra);
        assert_eq!(e2.difference(e1), Flags::B);
        assert_eq!(e2.complement(), Flags::A);
        assert_eq!(e1.complement(), Flags::B);
        assert_eq!(e1.symmetric_difference(e2), Flags::A | Flags::B | extra); // toggle
    }

    #[test]
    fn test_set_ops_exhaustive() {
        // Define a flag that contains gaps to help exercise edge-cases,
        // especially around "unknown" flags (e.g. ones outside of `all()`
        // `from_bits_unchecked`).
        // - when lhs and rhs both have different sets of unknown flags.
        // - unknown flags at both ends, and in the middle
        // - cases with "gaps".
        bitflags! {
            struct Test: u16 {
                // Intentionally no `A`
                const B = 0b000000010;
                // Intentionally no `C`
                const D = 0b000001000;
                const E = 0b000010000;
                const F = 0b000100000;
                const G = 0b001000000;
                // Intentionally no `H`
                const I = 0b100000000;
            }
        }
        let iter_test_flags =
            || (0..=0b111_1111_1111).map(|bits| unsafe { Test::from_bits_unchecked(bits) });

        for a in iter_test_flags() {
            assert_eq!(
                a.complement(),
                Test::from_bits_truncate(!a.bits),
                "wrong result: !({:?})",
                a,
            );
            assert_eq!(a.complement(), !a, "named != op: !({:?})", a);
            for b in iter_test_flags() {
                // Check that the named operations produce the expected bitwise
                // values.
                assert_eq!(
                    a.union(b).bits,
                    a.bits | b.bits,
                    "wrong result: `{:?}` | `{:?}`",
                    a,
                    b,
                );
                assert_eq!(
                    a.intersection(b).bits,
                    a.bits & b.bits,
                    "wrong result: `{:?}` & `{:?}`",
                    a,
                    b,
                );
                assert_eq!(
                    a.symmetric_difference(b).bits,
                    a.bits ^ b.bits,
                    "wrong result: `{:?}` ^ `{:?}`",
                    a,
                    b,
                );
                assert_eq!(
                    a.difference(b).bits,
                    a.bits & !b.bits,
                    "wrong result: `{:?}` - `{:?}`",
                    a,
                    b,
                );
                // Note: Difference is checked as both `a - b` and `b - a`
                assert_eq!(
                    b.difference(a).bits,
                    b.bits & !a.bits,
                    "wrong result: `{:?}` - `{:?}`",
                    b,
                    a,
                );
                // Check that the named set operations are equivalent to the
                // bitwise equivalents
                assert_eq!(a.union(b), a | b, "named != op: `{:?}` | `{:?}`", a, b,);
                assert_eq!(
                    a.intersection(b),
                    a & b,
                    "named != op: `{:?}` & `{:?}`",
                    a,
                    b,
                );
                assert_eq!(
                    a.symmetric_difference(b),
                    a ^ b,
                    "named != op: `{:?}` ^ `{:?}`",
                    a,
                    b,
                );
                assert_eq!(a.difference(b), a - b, "named != op: `{:?}` - `{:?}`", a, b,);
                // Note: Difference is checked as both `a - b` and `b - a`
                assert_eq!(b.difference(a), b - a, "named != op: `{:?}` - `{:?}`", b, a,);
                // Verify that the operations which should be symmetric are
                // actually symmetric.
                assert_eq!(a.union(b), b.union(a), "asymmetry: `{:?}` | `{:?}`", a, b,);
                assert_eq!(
                    a.intersection(b),
                    b.intersection(a),
                    "asymmetry: `{:?}` & `{:?}`",
                    a,
                    b,
                );
                assert_eq!(
                    a.symmetric_difference(b),
                    b.symmetric_difference(a),
                    "asymmetry: `{:?}` ^ `{:?}`",
                    a,
                    b,
                );
            }
        }
    }

    #[test]
    fn test_set() {
        let mut e1 = Flags::A | Flags::C;
        e1.set(Flags::B, true);
        e1.set(Flags::C, false);

        assert_eq!(e1, Flags::A | Flags::B);
    }

    #[test]
    fn test_assignment_operators() {
        let mut m1 = Flags::empty();
        let e1 = Flags::A | Flags::C;
        // union
        m1 |= Flags::A;
        assert_eq!(m1, Flags::A);
        // intersection
        m1 &= e1;
        assert_eq!(m1, Flags::A);
        // set difference
        m1 -= m1;
        assert_eq!(m1, Flags::empty());
        // toggle
        m1 ^= e1;
        assert_eq!(m1, e1);
    }

    #[test]
    fn test_const_fn() {
        const _M1: Flags = Flags::empty();

        const M2: Flags = Flags::A;
        assert_eq!(M2, Flags::A);

        const M3: Flags = Flags::C;
        assert_eq!(M3, Flags::C);
    }

    #[test]
    fn test_extend() {
        let mut flags;

        flags = Flags::empty();
        flags.extend([].iter().cloned());
        assert_eq!(flags, Flags::empty());

        flags = Flags::empty();
        flags.extend([Flags::A, Flags::B].iter().cloned());
        assert_eq!(flags, Flags::A | Flags::B);

        flags = Flags::A;
        flags.extend([Flags::A, Flags::B].iter().cloned());
        assert_eq!(flags, Flags::A | Flags::B);

        flags = Flags::B;
        flags.extend([Flags::A, Flags::ABC].iter().cloned());
        assert_eq!(flags, Flags::ABC);
    }

    #[test]
    fn test_from_iterator() {
        assert_eq!([].iter().cloned().collect::<Flags>(), Flags::empty());
        assert_eq!(
            [Flags::A, Flags::B].iter().cloned().collect::<Flags>(),
            Flags::A | Flags::B
        );
        assert_eq!(
            [Flags::A, Flags::ABC].iter().cloned().collect::<Flags>(),
            Flags::ABC
        );
    }

    #[test]
    fn test_lt() {
        let mut a = Flags::empty();
        let mut b = Flags::empty();

        assert!(!(a < b) && !(b < a));
        b = Flags::B;
        assert!(a < b);
        a = Flags::C;
        assert!(!(a < b) && b < a);
        b = Flags::C | Flags::B;
        assert!(a < b);
    }

    #[test]
    fn test_ord() {
        let mut a = Flags::empty();
        let mut b = Flags::empty();

        assert!(a <= b && a >= b);
        a = Flags::A;
        assert!(a > b && a >= b);
        assert!(b < a && b <= a);
        b = Flags::B;
        assert!(b > a && b >= a);
        assert!(a < b && a <= b);
    }

    fn hash<T: Hash>(t: &T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }

    #[test]
    fn test_hash() {
        let mut x = Flags::empty();
        let mut y = Flags::empty();
        assert_eq!(hash(&x), hash(&y));
        x = Flags::all();
        y = Flags::ABC;
        assert_eq!(hash(&x), hash(&y));
    }

    #[test]
    fn test_default() {
        assert_eq!(Flags::empty(), Flags::default());
    }

    #[test]
    fn test_debug() {
        assert_eq!(format!("{:?}", Flags::A | Flags::B), "A | B");
        assert_eq!(format!("{:?}", Flags::empty()), "(empty)");
        assert_eq!(format!("{:?}", Flags::ABC), "A | B | C | ABC");
        let extra = unsafe { Flags::from_bits_unchecked(0xb8) };
        assert_eq!(format!("{:?}", extra), "0xb8");
        assert_eq!(format!("{:?}", Flags::A | extra), "A | 0xb8");

        assert_eq!(
            format!("{:?}", Flags::ABC | extra),
            "A | B | C | ABC | 0xb8"
        );

        assert_eq!(format!("{:?}", EmptyFlags::empty()), "(empty)");
    }

    #[test]
    fn test_binary() {
        assert_eq!(format!("{:b}", Flags::ABC), "111");
        assert_eq!(format!("{:#b}", Flags::ABC), "0b111");
        let extra = unsafe { Flags::from_bits_unchecked(0b1010000) };
        assert_eq!(format!("{:b}", Flags::ABC | extra), "1010111");
        assert_eq!(format!("{:#b}", Flags::ABC | extra), "0b1010111");
    }

    #[test]
    fn test_octal() {
        assert_eq!(format!("{:o}", LongFlags::LONG_A), "177777");
        assert_eq!(format!("{:#o}", LongFlags::LONG_A), "0o177777");
        let extra = unsafe { LongFlags::from_bits_unchecked(0o5000000) };
        assert_eq!(format!("{:o}", LongFlags::LONG_A | extra), "5177777");
        assert_eq!(format!("{:#o}", LongFlags::LONG_A | extra), "0o5177777");
    }

    #[test]
    fn test_lowerhex() {
        assert_eq!(format!("{:x}", LongFlags::LONG_A), "ffff");
        assert_eq!(format!("{:#x}", LongFlags::LONG_A), "0xffff");
        let extra = unsafe { LongFlags::from_bits_unchecked(0xe00000) };
        assert_eq!(format!("{:x}", LongFlags::LONG_A | extra), "e0ffff");
        assert_eq!(format!("{:#x}", LongFlags::LONG_A | extra), "0xe0ffff");
    }

    #[test]
    fn test_upperhex() {
        assert_eq!(format!("{:X}", LongFlags::LONG_A), "FFFF");
        assert_eq!(format!("{:#X}", LongFlags::LONG_A), "0xFFFF");
        let extra = unsafe { LongFlags::from_bits_unchecked(0xe00000) };
        assert_eq!(format!("{:X}", LongFlags::LONG_A | extra), "E0FFFF");
        assert_eq!(format!("{:#X}", LongFlags::LONG_A | extra), "0xE0FFFF");
    }

    mod submodule {
        bitflags! {
            pub struct PublicFlags: i8 {
                const X = 0;
            }

            struct PrivateFlags: i8 {
                const Y = 0;
            }
        }

        #[test]
        fn test_private() {
            let _ = PrivateFlags::Y;
        }
    }

    #[test]
    fn test_public() {
        let _ = submodule::PublicFlags::X;
    }

    mod t1 {
        mod foo {
            pub type Bar = i32;
        }

        bitflags! {
            /// baz
            struct Flags: foo::Bar {
                const A = 0b00000001;
                #[cfg(foo)]
                const B = 0b00000010;
                #[cfg(foo)]
                const C = 0b00000010;
            }
        }
    }

    #[test]
    fn test_in_function() {
        bitflags! {
           struct Flags: u8 {
                const A = 1;
                #[cfg(any())] // false
                const B = 2;
            }
        }
        assert_eq!(Flags::all(), Flags::A);
        assert_eq!(format!("{:?}", Flags::A), "A");
    }

    #[test]
    fn test_deprecated() {
        bitflags! {
            pub struct TestFlags: u32 {
                #[deprecated(note = "Use something else.")]
                const ONE = 1;
            }
        }
    }

    #[test]
    fn test_pub_crate() {
        mod module {
            bitflags! {
                pub (crate) struct Test: u8 {
                    const FOO = 1;
                }
            }
        }

        assert_eq!(module::Test::FOO.bits(), 1);
    }

    #[test]
    fn test_pub_in_module() {
        mod module {
            mod submodule {
                bitflags! {
                    // `pub (in super)` means only the module `module` will
                    // be able to access this.
                    pub (in super) struct Test: u8 {
                        const FOO = 1;
                    }
                }
            }

            mod test {
                // Note: due to `pub (in super)`,
                // this cannot be accessed directly by the testing code.
                pub(super) fn value() -> u8 {
                    super::submodule::Test::FOO.bits()
                }
            }

            pub fn value() -> u8 {
                test::value()
            }
        }

        assert_eq!(module::value(), 1)
    }

    #[test]
    fn test_zero_value_flags() {
        bitflags! {
            struct Flags: u32 {
                const NONE = 0b0;
                const SOME = 0b1;
            }
        }

        assert!(Flags::empty().contains(Flags::NONE));
        assert!(Flags::SOME.contains(Flags::NONE));
        assert!(Flags::NONE.is_empty());

        assert_eq!(format!("{:?}", Flags::empty()), "NONE");
        assert_eq!(format!("{:?}", Flags::SOME), "SOME");
    }

    #[test]
    fn test_empty_bitflags() {
        bitflags! {}
    }

    #[test]
    fn test_u128_bitflags() {
        bitflags! {
            struct Flags128: u128 {
                const A = 0x0000_0000_0000_0000_0000_0000_0000_0001;
                const B = 0x0000_0000_0000_1000_0000_0000_0000_0000;
                const C = 0x8000_0000_0000_0000_0000_0000_0000_0000;
                const ABC = Self::A.bits | Self::B.bits | Self::C.bits;
            }
        }

        assert_eq!(Flags128::ABC, Flags128::A | Flags128::B | Flags128::C);
        assert_eq!(Flags128::A.bits, 0x0000_0000_0000_0000_0000_0000_0000_0001);
        assert_eq!(Flags128::B.bits, 0x0000_0000_0000_1000_0000_0000_0000_0000);
        assert_eq!(Flags128::C.bits, 0x8000_0000_0000_0000_0000_0000_0000_0000);
        assert_eq!(
            Flags128::ABC.bits,
            0x8000_0000_0000_1000_0000_0000_0000_0001
        );
        assert_eq!(format!("{:?}", Flags128::A), "A");
        assert_eq!(format!("{:?}", Flags128::B), "B");
        assert_eq!(format!("{:?}", Flags128::C), "C");
        assert_eq!(format!("{:?}", Flags128::ABC), "A | B | C | ABC");
    }

    #[test]
    fn test_serde_bitflags_serialize() {
        let flags = SerdeFlags::A | SerdeFlags::B;

        let serialized = serde_json::to_string(&flags).unwrap();

        assert_eq!(serialized, r#"{"bits":3}"#);
    }

    #[test]
    fn test_serde_bitflags_deserialize() {
        let deserialized: SerdeFlags = serde_json::from_str(r#"{"bits":12}"#).unwrap();

        let expected = SerdeFlags::C | SerdeFlags::D;

        assert_eq!(deserialized.bits, expected.bits);
    }

    #[test]
    fn test_serde_bitflags_roundtrip() {
        let flags = SerdeFlags::A | SerdeFlags::B;

        let deserialized: SerdeFlags =
            serde_json::from_str(&serde_json::to_string(&flags).unwrap()).unwrap();

        assert_eq!(deserialized.bits, flags.bits);
    }

    bitflags! {
        #[derive(serde::Serialize, serde::Deserialize)]
        struct SerdeFlags: u32 {
            const A = 1;
            const B = 2;
            const C = 4;
            const D = 8;
        }
    }
}

use super::*;

impl<C> AssociativeMonoidRule<C> for () {}
impl<C> MonoidRule<C> for () {
    fn apply(mut string: Vec<C>, letter: C) -> Vec<C> {string.push(letter); string}
    fn apply_many(mut string1: Vec<C>, mut string2: Vec<C>) -> Vec<C> {
        string1.append(&mut string2); string1
    }
    fn apply_iter<I:Iterator<Item=C>>(mut string: Vec<C>, letters: I) -> Vec<C> {
        string.extend(letters); string
    }
}

///
///Wraps a type `T` and symbolically inverts elements.
///
///Used for constructing [FreeGroup]
///
pub enum FreeInv<T> {
    ///Wraps an instance of type `T`
    Id(T),
    ///The symbolic inverse of an object of type `T`
    Inv(T)
}

impl<T> FreeInv<T> {
    pub fn is_inv(&self) -> bool {
        match self { Self::Inv(_) => true, _ => false }
    }
    pub fn is_id(&self) -> bool {
        match self { Self::Id(_) => true, _ => false }
    }
}

///Multiplication of [FreeInv] elements using concatenation with inverse cancellation
pub struct InvRule;

impl<T> MonoidRule<FreeInv<T>> for InvRule {
    fn apply(mut string: Vec<FreeInv<T>>, letter: FreeInv<T>) -> Vec<FreeInv<T>> {
        if string.last().map_or(false, |last| letter.is_inv() ^ last.is_inv()) {
            string.pop();
        } else {
            string.push(letter);
        }
        string
    }
}

impl<T> InvMonoidRule<FreeInv<T>> for InvRule {
    fn invert(letter: FreeInv<T>) -> FreeInv<T> { letter.inv() }
}

impl<T> Inv for FreeInv<T> {
    type Output = Self;
    fn inv(self) -> Self {
        match self {
            Self::Id(x) => Self::Inv(x),
            Self::Inv(x) => Self::Id(x)
        }
    }
}

impl<T> Mul for FreeInv<T> {
    type Output = FreeGroup<T>;
    fn mul(self, rhs:Self) -> FreeGroup<T> { FreeGroup::from(self) * rhs }
}

impl<T> Div for FreeInv<T> {
    type Output = FreeGroup<T>;
    fn div(self, rhs:Self) -> FreeGroup<T> { self * rhs.inv() }
}

impl<T> Mul<FreeGroup<T>> for FreeInv<T> {
    type Output = FreeGroup<T>;
    fn mul(self, rhs:FreeGroup<T>) -> FreeGroup<T> { FreeGroup::from(self) * rhs }
}

impl<T> Div<FreeGroup<T>> for FreeInv<T> {
    type Output = FreeGroup<T>;
    fn div(self, rhs:FreeGroup<T>) -> FreeGroup<T> { FreeGroup::from(self) / rhs }
}

///
///A [monoid](MulMonoid) constructed from free multiplication of elements of a set
///
///Concretely, given a set `C`, we construct the free-monoid of `C` as the set of all finite lists
///of members of `C` where multiplication is given by concatenation. In other words, it's basically
///[`Vec<C>`](Vec) but with `a*b == {a.append(&mut b); a}`.
///
pub type FreeMonoid<C> = MonoidalString<C,()>;

///A [FreeMonoid], but where each letter can be inverted
pub type FreeGroup<C> = MonoidalString<FreeInv<C>,InvRule>;

///
///Represents a free symbol raised to some power
///
///This is used in [FreePowMonoid] mainly as a way to compress the size-footprint of a
///[FreeMonoid] by combining repeated elements under an integral exponent, but the option for more
///exotic exponents is also available
///
#[derive(Derivative)]
#[derivative(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub struct FreePow<C:Eq,P>(pub C,pub P);

impl<C:Eq,P:One+Neg<Output=P>> From<FreeInv<C>> for FreePow<C,P> {
    fn from(inv: FreeInv<C>) -> Self {
        match inv {
            FreeInv::Id(x) => FreePow(x,P::one()),
            FreeInv::Inv(x) => FreePow(x,-P::one()),
        }
    }
}

///Multiplication between [FreePow] elements using addition of exponents on equal bases
pub struct PowRule;

impl<C:Eq,P:Add<Output=P>+AddAssociative> AssociativeMonoidRule<FreePow<C,P>> for PowRule {}
impl<C:Eq,P:Add<Output=P>> MonoidRule<FreePow<C,P>> for PowRule {
    fn apply(mut string: Vec<FreePow<C,P>>, letter: FreePow<C,P>) -> Vec<FreePow<C,P>> {
        if string.last().map_or(false, |l| l.0==letter.0) {
            let last = string.pop().unwrap();
            let last = FreePow(letter.0, last.1 + letter.1);
            if !last.1._is_zero() { string.push(last); }
        } else {
            string.push(letter);
        }
        string
    }
}

impl<C:Eq,P:Add<Output=P>+Neg<Output=P>> InvMonoidRule<FreePow<C,P>> for PowRule {
    fn invert(free: FreePow<C,P>) -> FreePow<C,P> { free.inv() }
}

impl<C:Eq,P:One> From<C> for FreePow<C,P> { fn from(c:C) -> Self { (c,P::one()).into() } }
impl<C:Eq,P> From<(C,P)> for FreePow<C,P> { fn from((c,z):(C,P)) -> Self { FreePow(c,z) } }

impl<C:Eq,P:Neg<Output=P>> Inv for FreePow<C,P> {
    type Output = Self;
    fn inv(self) -> Self { FreePow(self.0, -self.1) }
}

impl<C:Eq,P:Add<Output=P>> Mul for FreePow<C,P> {
    type Output = FreePowMonoid<C,P>;
    fn mul(self, rhs:Self) -> FreePowMonoid<C,P> { FreePowMonoid::from(self) * rhs }
}

impl<C:Eq,P:Add<Output=P>+One> Mul<C> for FreePow<C,P> {
    type Output = FreePowMonoid<C,P>;
    fn mul(self, rhs:C) -> FreePowMonoid<C,P> { self * Self::from(rhs) }
}

impl<C:Eq,P:Add<Output=P>> Mul<FreePowMonoid<C,P>> for FreePow<C,P> {
    type Output = FreePowMonoid<C,P>;
    fn mul(self, rhs:FreePowMonoid<C,P>) -> FreePowMonoid<C,P> { FreePowMonoid::from(self) * rhs }
}

impl<C:Eq,P:Add<Output=P>+Neg<Output=P>> Div for FreePow<C,P> {
    type Output = FreePowMonoid<C,P>;
    fn div(self, rhs:Self) -> FreePowMonoid<C,P> { self * rhs.inv() }
}

impl<C:Eq,P:Add<Output=P>+One+Neg<Output=P>> Div<C> for FreePow<C,P> {
    type Output = FreePowMonoid<C,P>;
    fn div(self, rhs:C) -> FreePowMonoid<C,P> { self / Self::from(rhs) }
}

impl<C:Eq,P:Add<Output=P>+Neg<Output=P>> Div<FreePowMonoid<C,P>> for FreePow<C,P> {
    type Output = FreePowMonoid<C,P>;
    fn div(self, rhs:FreePowMonoid<C,P>) -> FreePowMonoid<C,P> { FreePowMonoid::from(self) / rhs }
}

///A [FreeModule] or [FreeGroup], but where repeated letters are grouped together using exponents
pub type FreePowMonoid<C,P> = MonoidalString<FreePow<C,P>,PowRule>;


///Multiplication of terms using a type's intrinsic [addition](Add) operation
pub struct AddRule;
///Multiplication of terms using a type's intrinsic [multiplication](Mul) operation
pub struct MulRule;

impl<R,T:Add<Output=T>> AlgebraRule<R,T> for AddRule {
    fn apply(t1:T, t2:T) -> (Option<R>,T) { (None, t1+t2) }
}
impl<R,T:Add<Output=T>+Zero> UnitalAlgebraRule<R,T> for AddRule {
    fn one() -> T { T::zero() }
    fn is_one(t:&T) -> bool { t.is_zero() }
}

impl<R,T:Mul<Output=T>> AlgebraRule<R,T> for MulRule {
    fn apply(t1:T, t2:T) -> (Option<R>,T) { (None, t1*t2) }
}
impl<R,T:Mul<Output=T>+One+PartialEq> UnitalAlgebraRule<R,T> for MulRule {
    fn one() -> T { T::one() }
    fn is_one(t:&T) -> bool { t.is_one() }
}

///A [FreeModule] over some monoid, but with a multiplication between elements given using the monoid operation
pub type MonoidRing<R,M> = ModuleString<R,M,MulRule>;

///
///A [module](RingModule) over a ring constructed from free addition scalar-multiplication of elements of a set
///
///Concretely, given a set `T` and ring `R`, we construct the free-module of `T` over `R` as
///the set of linear combination over elements in `T` where the scalars are in `R`.
///
pub type FreeModule<R,T> = ModuleString<R,T,!>;

///
///A [module](RingModule) over a ring constructed from free multiplication and addition of elements of a set
///
///Concretely, this is a [MonoidRing] over `R` of the [FreeMonoid] over `T`. However, this is
///effectively just the set of polynomials with coeffients in `R` where the variables are all the
///elements of `T` (and where we don't assume associativity or commutivity of the
///variables).
///
pub type FreeAlgebra<R,T> = MonoidRing<R,FreeMonoid<T>>;

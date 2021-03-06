//!
//!Contains [ModuleString] and the types and traits relevant to its system
//!
//!For more information see the struct-level docs
//!

use super::*;

use std::collections::hash_map;

///
///Creates free-arithmetic constructions based upon order-invariant addition of terms of type `C` with
///multiplication with scalar coefficients of type `R`
///
///# Basic Construction
///
///Given type parameters `R`, `T`, and `A`, the construction goes as follows:
/// * Internally, each instance contains a [map](HashMap) from instances of `T` to coefficients from `R`
/// * Addition is then implemented by merging the arguments' [HashMaps](HashMap) where repeated terms
///  have their coefficients added together
/// * Multiplication with `R` values is applied by each term's coefficient with the given scalar
/// * Multiplication between terms is performed by multiplying the coefficients and apply the rule
///  defined by type `A`
/// * Then, full multiplication is defined by distributing over each pair of terms and summing
///
///With this system, using different `A` rules and `T`/`R` types, a number of variants can be defined:
/// * [`FreeModule<R,T>`](FreeModule) defines a [vector-space](VectorSpace) over `R` using the default
///  addition and `R`-multiplication without having a multiplication rule
/// * [`FreeAlgebra<R,T>`](FreeAlgebra) is a `FreeModule<R,T>` but where the multiplication rule
///  between terms is built as with a [`FreeMonoid<T>`](FreeMonoid)
/// * [`MonoidRing<R,T>`](MonoidRing) is a `FreeModule<R,T>` but where the multiplication rule
///  for terms simply uses the intrinsic multiplication of `T`
///
///# Other `impls`
///
///In addition to the basic arithmetic operations, [ModuleString] implements a number of other
///notable traits depending on the type arguments:
/// * [Sub], [SubAssign], [Neg], etc. These apply by negating each coefficient whenever `R` is negatable
/// * [Index] gets coefficient references from a term reference
/// * [MulAssociative] and [MulCommutative] whenever `R` is and `A` implements [AssociativeMonoidRule]
///  or [CommutativeMonoidRule]
/// * [Distributive] whenever `R` is
/// * [Extend], [FromIterator], and [Sum] all are implemented using repeated addition.
///
///# A note on [IterMut]
///
///The method [ModuleString.iter_mut()] *will* give an valid iterator over mutable references to each
///element of the object, but it is of note that because of the nature of some of the [`AlgebraRule`]'s,
///the method will reallocate internal storage and iterate through *every* element,
///even if dropped.
///
///The reason for this is that if it did not, it would be possible to modify a ModuleString into
///an invalid state. Hence, to mitigate this, the iterator must remultiply every element again as it
///iterates over the references.
///
#[derive(Derivative)]
#[derivative(Clone(clone_from="true"))]
#[derivative(Default(bound=""))]
#[derivative(PartialEq, Eq, Hash)]
#[derivative(Debug="transparent")]
pub struct ModuleString<R,T:Hash+Eq,A:?Sized> {
    #[derivative(Hash(bound="HashMap<T,R>:Hash"))]
    #[derivative(Default(value="HashMap::with_capacity(0)"))]
    terms: HashMap<T,R>,

    #[derivative(PartialEq="ignore", Hash="ignore")]
    #[derivative(Debug="ignore")]
    rule: PhantomData<Box<A>>
}

///
///Formats the [ModuleString] as a sequence of factors separated by `+`'s
///
///# Examples
///```
///use maths_traits::algebra::{One, Zero};
///use free_algebra::{FreeAlgebra, FreeMonoid};
///
///let one = FreeMonoid::one();
///let x:FreeMonoid<_> = 'x'.into();
///let y:FreeMonoid<_> = 'y'.into();
///
///let mut p = FreeAlgebra::zero();
///assert_eq!(format!("{}", p), "0");
///
///p += (1.0, one.clone());
///assert_eq!(format!("{}", p), "1");
///
///p += (2.5, one);
///assert_eq!(format!("{}", p), "3.5");
///
///p += x;
///assert!(format!("{}", p) == "(3.5 + x)" || format!("{}", p) == "(x + 3.5)");
///
///p *= (1.0, y);
///assert!(format!("{}", p) == "(3.5*y + x*y)" || format!("{}", p) == "(x*y + 3.5*y)");
///assert!(format!("{:#}", p) == "(3.5y + xy)" || format!("{:#}", p) == "(xy + 3.5y)");
///
///```
///
impl<R:Display,T:Hash+Eq+Display,A:?Sized> Display for ModuleString<R,T,A> {
    fn fmt(&self, f: &mut Formatter) -> ::std::fmt::Result {
        if self.len() == 0 {
            //try to print 0 as formatted by R
            match R::_zero() {
                Some(zero) => write!(f, "{}", zero)?,
                None => write!(f, "{}", 0)?,
            }
        } else {
            //put parens around sums
            if self.len() > 1 { write!(f, "(")?; }

            //use specialization to determine if a T is one
            trait IsTOne<R,T> { fn _is_t_one(t:&T) -> bool; }
            impl<R,T,A:?Sized> IsTOne<R,T> for A { default fn _is_t_one(_:&T) -> bool {false} }
            impl<R,T,A:UnitalAlgebraRule<R,T>+?Sized> IsTOne<R,T> for A { fn _is_t_one(t:&T) -> bool {A::is_one(t)} }

            //use specialization to determine if a R is one
            trait IsOne { fn _is_one(&self) -> bool; }
            impl<R> IsOne for R { default fn _is_one(&self) -> bool {false} }
            impl<R:One+PartialEq> IsOne for R { fn _is_one(&self) -> bool { self.is_one() } }

            //print every term in a sum
            let mut first = true;
            for (r,t) in self.iter() {

                //add the plus sign to the previous sum
                if !first {
                    write!(f, " + ")?;
                } else {
                    first = false;
                }

                //if the term isn't one, write the term
                if !<A as IsTOne<R,T>>::_is_t_one(t) {
                    //if the coeffient is one, don't write it
                    if (*r)._is_one() {
                        if f.alternate() {
                            write!(f, "{:#}", t)?;
                        } else {
                            write!(f, "{}", t)?;
                        }
                    } else {
                        if f.alternate() {
                            write!(f, "{:#}{:#}", r, t)?;
                        } else {
                            write!(f, "{}*{}", r, t)?;
                        }
                    }
                } else {
                    write!(f, "{}", r)?;
                }
            }

            //put parens around sums
            if self.len() > 1 { write!(f, ")")?; }
        }

        //success
        Ok(())
    }
}

///Iterates over references to the terms and coefficients of a [ModuleString]
pub type Iter<'a,R,T> = Map<hash_map::Iter<'a,T,R>, fn((&'a T,&'a R)) -> (&'a R,&'a T)>;

///Iterates over terms and coefficients of a [ModuleString]
pub type IntoIter<R,T> = Map<hash_map::IntoIter<T,R>, fn((T,R)) -> (R,T)>;

///
///Iterates over mutable references to the terms and coefficients of a [ModuleString]
///
///Note that this causes a reallocation of the internal [HashMap] since it's possible that an
///element mutation could create an illegal state if not reconstructed from the sums of the mutated
///terms. (For example, one could easily mutate two terms to have the same `T` value and thus
///potentially overwrite the coeffient of one of them if not handled properly)
///
pub struct IterMut<'a, R:AddAssign, T:Hash+Eq, A:?Sized> {
    dest_ref: &'a mut ModuleString<R,T,A>,
    next: Option<(R,T)>,
    iter: IntoIter<R,T>
}

impl<'a,T:Hash+Eq,R:AddAssign,A:?Sized> FusedIterator for IterMut<'a,R,T,A> {}
impl<'a,T:Hash+Eq,R:AddAssign,A:?Sized> ExactSizeIterator for IterMut<'a,R,T,A> {}
impl<'a,T:Hash+Eq,R:AddAssign,A:?Sized> Iterator for IterMut<'a,R,T,A> {
    type Item = (&'a mut R, &'a mut T);
    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|t| *self.dest_ref += t);
        self.next = self.iter.next();

        //we know that the given reference is lifetime 'a because in order for next to be dropped,
        //either self must be borrowed mutably again or dropped, which cannot happen while the reference lives
        self.next.as_mut().map(|t| unsafe {&mut *(t as *mut (R,T))} ).map(|t| (&mut t.0, &mut t.1))
    }

    fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }
}

impl<'a,T:Hash+Eq,R:AddAssign,A:?Sized> Drop for IterMut<'a,R,T,A> {
    fn drop(&mut self) {
        loop { if let None = self.next() {break;} }
    }
}

impl<T:Hash+Eq,R:One,A:?Sized> From<T> for ModuleString<R,T,A> {
    #[inline] fn from(t:T) -> Self {(R::one(),t).into()}
}

impl<T:Hash+Eq,R:One,A:?Sized> From<(R, T)> for ModuleString<R,T,A> {
    #[inline]
    fn from((r,t):(R,T)) -> Self {
        let mut m = HashMap::new();
        m.insert(t, r);
        ModuleString{terms:m, rule:PhantomData}
    }
}

impl<T:Hash+Eq,R,A:?Sized,I> Index<I> for ModuleString<R,T,A> where HashMap<T,R>:Index<I> {
    type Output = <HashMap<T,R> as Index<I>>::Output;
    #[inline] fn index(&self, i:I) -> &Self::Output {&self.terms[i]}
}

impl<T:Hash+Eq,R,A:?Sized> IntoIterator for ModuleString<R,T,A> {
    type Item = (R,T);
    type IntoIter = IntoIter<R,T>;
    #[inline] fn into_iter(self) -> IntoIter<R,T> { self.terms.into_iter().map(|(t,r)| (r,t)) }
}

impl<T:Hash+Eq,R,A:?Sized> ModuleString<R,T,A> {

    ///
    ///Returns the number of terms in this module element
    ///
    ///## Examples
    ///```
    ///use maths_traits::algebra::Zero;
    ///use free_algebra::FreeModule;
    ///
    ///let p = FreeModule::zero();
    ///let q = &p + (3.5, 'x');
    ///let r = &q + (2.0, 'y');
    ///let s = &r + (2.0, 'x'); //adds to first term without increasing length
    ///
    ///assert_eq!(p.len(), 0);
    ///assert_eq!(q.len(), 1);
    ///assert_eq!(r.len(), 2);
    ///assert_eq!(s.len(), 2);
    ///
    ///```
    ///
    pub fn len(&self) -> usize {self.terms.len()}

    ///
    ///Retrieves a reference to the coefficient of a term, if it is non-zero
    ///
    ///## Examples
    ///```
    ///use maths_traits::algebra::Zero;
    ///use free_algebra::FreeModule;
    ///
    ///let p = FreeModule::zero() + (3.5, 'x') + (2.0, 'y') + (1.0, 'z');
    ///
    ///assert_eq!(p.get_ref(&'x'), Some(&3.5));
    ///assert_eq!(p.get_ref(&'y'), Some(&2.0));
    ///assert_eq!(p.get_ref(&'z'), Some(&1.0));
    ///assert_eq!(p.get_ref(&'w'), None);
    ///
    ///```
    ///
    pub fn get_ref<Q:Hash+Eq>(&self, t: &Q) -> Option<&R> where T:Borrow<Q> {
        self.terms.get(t)
    }

    ///
    ///Clones a the coefficient of a term or returns [zero](Zero::zero()) if it doesn't exist
    ///
    ///## Examples
    ///```
    ///use maths_traits::algebra::Zero;
    ///use free_algebra::FreeModule;
    ///
    ///let p = FreeModule::zero() + (3.5, 'x') + (2.0, 'y') + (1.0, 'z');
    ///
    ///assert_eq!(p.get(&'x'), 3.5);
    ///assert_eq!(p.get(&'y'), 2.0);
    ///assert_eq!(p.get(&'z'), 1.0);
    ///assert_eq!(p.get(&'w'), 0.0);
    ///
    ///```
    ///
    pub fn get<Q:Hash+Eq>(&self, t: &Q) -> R where T:Borrow<Q>, R:Zero+Clone {
        self.terms.get(t).map_or_else(|| R::zero(), |r| r.clone())
    }

    ///Produces an iterator over references to the terms and references in this element
    pub fn iter<'a>(&'a self) -> Iter<'a,R,T> { self.terms.iter().map(|(t,r)| (r,t)) }

    ///
    ///Produces an iterator over mutable references to the terms and references in this element
    ///
    ///## Examples
    ///```
    ///use maths_traits::algebra::Zero;
    ///use free_algebra::FreeModule;
    ///
    ///let mut p = FreeModule::zero() + (3.5, 'x') + (2.0, 'y') + (1.0, 'z');
    ///for (_, t) in p.iter_mut() {
    ///    *t = 'a';
    ///}
    ///
    ///assert_eq!(p.get(&'x'), 0.0);
    ///assert_eq!(p.get(&'y'), 0.0);
    ///assert_eq!(p.get(&'z'), 0.0);
    ///assert_eq!(p.get(&'a'), 6.5);
    ///
    ///```
    ///
    pub fn iter_mut<'a>(&'a mut self) -> IterMut<'a,R,T,A> where R:AddAssign {
        let mut temp = Self { terms: HashMap::with_capacity(self.len()), rule:PhantomData };
        ::std::mem::swap(self, &mut temp);
        IterMut { dest_ref: self, next: None, iter: temp.into_iter() }
    }

    ///
    ///Computes the algebraic commutator `[a,b] = a*b - b*a`
    ///
    ///## Examples
    ///```
    ///use maths_traits::algebra::One;
    ///use free_algebra::{FreeAlgebra, FreeMonoid};
    ///
    ///let one = FreeMonoid::<char>::one();
    ///let x:FreeMonoid<_> = 'x'.into();
    ///let y:FreeMonoid<_> = 'y'.into();
    ///
    ///let p:FreeAlgebra<f32,_> = FreeAlgebra::one() + &x;
    ///let q:FreeAlgebra<f32,_> = FreeAlgebra::one() + &y;
    ///
    ///let commutator = p.commutator(q);
    ///
    ///assert_eq!(commutator.get(&one), 0.0);
    ///assert_eq!(commutator.get(&x), 0.0);
    ///assert_eq!(commutator.get(&y), 0.0);
    ///assert_eq!(commutator.get(&(&x*&y)), 1.0);
    ///assert_eq!(commutator.get(&(&y*&x)), -1.0);
    ///```
    ///
    ///A property of significance for this product is that when the arguments commute, the
    ///output is always `0`.
    ///
    ///```
    ///use maths_traits::algebra::{One, Zero};
    ///use free_algebra::{FreeAlgebra, FreeMonoid};
    ///
    ///let p = FreeAlgebra::<f32,_>::one() + FreeMonoid::from('x');
    ///
    ///let commutator = p.clone().commutator(p);
    ///assert!(commutator.is_zero());
    ///```
    ///
    ///
    ///
    pub fn commutator(self, rhs:Self) -> Self where Self:MulMagma+Sub<Output=Self> {
        self.clone()*rhs.clone() - rhs*self
    }

}

impl<T:Hash+Eq,R,A:?Sized> ModuleString<R,T,A> {

    fn insert_term<F:Fn(R)->R>(&mut self, rhs:(R,T), f:F) where R:AddAssign{
        if !rhs.0._is_zero() {
            let (r, t) = (f(rhs.0), rhs.1);
            if let Some(r2) = self.terms.get_mut(&t) {
                *r2 += r;

                //Note, we must be VERY careful here because simply doing r2._is_zero() will run
                //_is_zero() on the REFERENCE to the coefficient, not the coeffient itself
                if (*r2)._is_zero() { self.terms.remove(&t); }
            } else if !r._is_zero() {
                self.terms.insert(t, r);
            }
        }
    }

    fn insert<I:IntoIterator<Item=(R,T)>, F:Fn(R)->R+Copy>(&mut self, rhs:I, f:F) where R:AddAssign {
        rhs.into_iter().for_each(|term| self.insert_term(term, f));
    }

    ///removes all terms with a coeffient of zero
    fn clean(&mut self) { self.terms.retain(|_,r| !r._is_zero()); }
}

impl<T:Hash+Eq,R:AddAssign,A:?Sized> Extend<(R,T)> for ModuleString<R,T,A> {
    fn extend<I:IntoIterator<Item=(R,T)>>(&mut self, iter:I) { self.insert(iter, |r| r) }
}

impl<T:Hash+Eq,R:AddAssign,A:?Sized> FromIterator<(R,T)> for ModuleString<R,T,A> {
    fn from_iter<I:IntoIterator<Item=(R,T)>>(iter:I) -> Self {
        let mut from = Self::default();
        from.extend(iter);
        from
    }
}

impl<T:Hash+Eq,R:AddAssign+One,A:?Sized> FromIterator<T> for ModuleString<R,T,A> {
    fn from_iter<I:IntoIterator<Item=T>>(iter:I) -> Self {
        Self::from_iter(iter.into_iter().map(|t| (R::one(), t)))
    }
}

impl<T:Hash+Eq,R:AddAssign,A:?Sized> FromIterator<Self> for ModuleString<R,T,A> {
    fn from_iter<I:IntoIterator<Item=Self>>(iter:I) -> Self {
        Self::from_iter(iter.into_iter().flatten())
    }
}

impl<T:Hash+Eq,R:AddAssign,A:?Sized,K> Sum<K> for ModuleString<R,T,A> where Self:FromIterator<K> {
    fn sum<I:Iterator<Item=K>>(iter:I) -> Self { Self::from_iter(iter) }
}

impl<T:Hash+Eq,R,A:?Sized,K> Product<K> for ModuleString<R,T,A> where Self:Mul<K,Output=Self>+One {
    fn product<I:Iterator<Item=K>>(iter:I) -> Self { iter.into_iter().fold(Self::one(), |m,k| m*k) }
}

///Dictates a rule for how to multiply terms in a [ModuleString]
pub trait AlgebraRule<R,T> {
    ///Multiplies two terms together to produce another `T` and an optional coeffient
    fn apply(t1:T, t2:T) -> (Option<R>,T);
}

///An [AlgebraRule] that is evaluation order independent
pub trait AssociativeAlgebraRule<R,T>: AlgebraRule<R,T> {}
///An [AlgebraRule] that is order independent
pub trait CommutativeAlgebraRule<R,T>: AlgebraRule<R,T> {}

///An [AlgebraRule] where a term can be one
pub trait UnitalAlgebraRule<R,T>: AlgebraRule<R,T> {
    ///Creates a `T` with a value of one
    fn one() -> T;
    ///Determines if a `T` is equal to one
    fn is_one(t:&T) -> bool;
}

impl<T:Hash+Eq,R:AddAssociative,A:?Sized> AddAssociative for ModuleString<R,T,A> {}
impl<T:Hash+Eq,R:AddCommutative,A:?Sized> AddCommutative for ModuleString<R,T,A> {}
impl<T:Hash+Eq,R:MulAssociative,A:?Sized+AssociativeAlgebraRule<R,T>> MulAssociative for ModuleString<R,T,A> {}
impl<T:Hash+Eq,R:MulCommutative,A:?Sized+CommutativeAlgebraRule<R,T>> MulCommutative for ModuleString<R,T,A> {}

impl<T:Hash+Eq,R:Distributive,A:?Sized> Distributive for ModuleString<R,T,A> {}

impl<T:Hash+Eq,R:AddAssign,A:?Sized> AddAssign for ModuleString<R,T,A> {
    fn add_assign(&mut self, rhs:Self) {self.insert(rhs, |r| r)}
}
impl<T:Hash+Eq,R:AddAssign+Neg<Output=R>,A:?Sized> SubAssign for ModuleString<R,T,A> {
    fn sub_assign(&mut self, rhs:Self) {self.insert(rhs, |r| -r)}
}

impl<T:Hash+Eq,R:AddAssign,A:?Sized> AddAssign<(R,T)> for ModuleString<R,T,A> {
    fn add_assign(&mut self, rhs:(R,T)) {self.insert_term(rhs, |r| r)}
}
impl<T:Hash+Eq,R:AddAssign+Neg<Output=R>,A:?Sized> SubAssign<(R,T)> for ModuleString<R,T,A> {
    fn sub_assign(&mut self, rhs:(R,T)) {self.insert_term(rhs, |r| -r)}
}

impl<T:Hash+Eq,R:AddAssign+One,A:?Sized> AddAssign<T> for ModuleString<R,T,A> {
    fn add_assign(&mut self, rhs:T) {*self+=(R::one(),rhs)}
}
impl<T:Hash+Eq,R:AddAssign+Neg<Output=R>+One,A:?Sized> SubAssign<T> for ModuleString<R,T,A> {
    fn sub_assign(&mut self, rhs:T) {*self-=(R::one(),rhs)}
}

impl<T:Hash+Eq,R:MulAssign+Clone,A:?Sized> MulAssign<R> for ModuleString<R,T,A> {
    fn mul_assign(&mut self, rhs:R) {
        for r2 in self.terms.values_mut() { *r2 *= rhs.clone() }
        self.clean();
    }
}
impl<T:Hash+Eq,R:DivAssign+Clone,A:?Sized> DivAssign<R> for ModuleString<R,T,A> {
    fn div_assign(&mut self, rhs:R) {
        for r2 in self.terms.values_mut() { *r2 /= rhs.clone() }
        self.clean();
    }
}

impl_arith!(impl<T,R,A> AddAssign<&Self>.add_assign for ModuleString<R,T,A> where T:Hash+Eq,A:?Sized);
impl_arith!(impl<T,R,A> SubAssign<&Self>.sub_assign for ModuleString<R,T,A> where T:Hash+Eq,A:?Sized);

impl_arith!(impl<T,R,A> AddAssign<&T>.add_assign for ModuleString<R,T,A> where T:Hash+Eq,A:?Sized);
impl_arith!(impl<T,R,A> SubAssign<&T>.sub_assign for ModuleString<R,T,A> where T:Hash+Eq,A:?Sized);

impl_arith!(impl<T,R,A> MulAssign<&R>.mul_assign for ModuleString<R,T,A> where T:Hash+Eq,A:?Sized);
impl_arith!(impl<T,R,A> DivAssign<&R>.div_assign for ModuleString<R,T,A> where T:Hash+Eq,A:?Sized);

impl_arith!(impl<T,R,A> Add.add with AddAssign.add_assign for ModuleString<R,T,A> where T:Hash+Eq, A:?Sized);
impl_arith!(impl<T,R,A> Sub.sub with SubAssign.sub_assign for ModuleString<R,T,A> where T:Hash+Eq, A:?Sized);
impl_arith!(impl<T,R,A> Mul.mul with MulAssign.mul_assign for ModuleString<R,T,A> where T:Hash+Eq, A:?Sized);
impl_arith!(impl<T,R,A> Div.div with DivAssign.div_assign for ModuleString<R,T,A> where T:Hash+Eq, A:?Sized);

impl<T:Hash+Eq,R:Neg,A:?Sized> Neg for ModuleString<R,T,A> {
    type Output = ModuleString<R::Output,T,A>;
    fn neg(self) -> Self::Output {
        ModuleString {
            terms: self.terms.into_iter().map(|(t,r)| (t,-r)).filter(|(_,r)| r._is_zero()).collect(),
            rule: PhantomData
        }
    }
}

impl<'a,T:Hash+Eq,R:Neg,A:?Sized> Neg for &'a ModuleString<R,T,A> where ModuleString<R,T,A>:Clone {
    type Output = ModuleString<R::Output,T,A>;
    #[inline] fn neg(self) -> Self::Output {-(*self).clone()}
}

impl<T:Hash+Eq,R:AddAssign,A:?Sized> Zero for ModuleString<R,T,A> {
    #[inline] fn zero() -> Self {Default::default()}
    #[inline] fn is_zero(&self) -> bool {self.terms.len()==0}
}

impl<T:Clone+Hash+Eq,R:PartialEq+UnitalSemiring,A:UnitalAlgebraRule<R,T>+?Sized> One for ModuleString<R,T,A> {
    #[inline] fn one() -> Self { A::one().into() }
    #[inline] fn is_one(&self) -> bool {
        if self.terms.len()==1 {
            let term = self.iter().next().unwrap();
            term.0.is_one() && A::is_one(term.1)
        } else {
            false
        }
    }
}


impl<T:Hash+Eq+Clone,R:MulMagma+AddMagma,A:?Sized+AlgebraRule<R,T>> MulAssign<(R,T)> for ModuleString<R,T,A> {
    fn mul_assign(&mut self, (r1,t1): (R,T)) {
        let mut temp = HashMap::with_capacity(0);
        ::std::mem::swap(&mut self.terms, &mut temp);
        *self = temp.into_iter().map(
            |(t, r)| {
                let (coeff, t2) = A::apply(t, t1.clone());
                match coeff {
                    Some(r2) => ((r*r1.clone())*r2, t2),
                    None => (r*r1.clone(), t2),
                }
            }
        ).sum();
    }
}

impl<T:Hash+Eq+Clone,R:Semiring,A:?Sized+AlgebraRule<R,T>> MulAssign for ModuleString<R,T,A> {
    fn mul_assign(&mut self, rhs:Self) {
        *self = rhs.terms.into_iter().map(|(t, r)| self.clone() * (r,t)).sum();
    }
}

impl<Z,R,T,A> Pow<Z> for ModuleString<R,T,A> where
    Z:Natural,
    T:Hash+Eq+Clone,
    R:PartialEq+UnitalSemiring,
    A:?Sized+UnitalAlgebraRule<R,T>+AssociativeAlgebraRule<R,T>
{
    type Output = Self;
    fn pow(self, p:Z) -> Self { repeated_squaring(self, p) }
}

# Chapter 5: Fixed Income

5
Fixed Income
5.1 Generalities
Zero-Coupon Bonds
A promise of being paid $1 at the maturity time T can be regarded as an asset,
which has some worth at time t before T . This asset is called a (zero-coupon)
discount bond. Let its price at time 0 ≤ t ≤ T be P(t, T ).T h e nP(T , T ) = 1.
The yield of a discount bond is deﬁned as 1
R(t, T ) =− ln(P(t, T ))
T − t (5.1)
and has the meaning of an average interest rate over the period of time T − t .
The higher the bond price at time t , the lower the yield R(t, T ) and vice versa.
Below we refer to a zero-coupon bond with a $1 principal and maturity T as
a T -bond.
Bonds with Coupons
In practice, a bond usually pays not only its principal at maturity T ,b u t
also makes smaller coupon payments before maturity. Consider a bond that
1More precisely, this deﬁnition assumes continuous compounding. For periodic compounding at n discrete
times Ti = T0 + i δ, i = 1,..., n, the yield between t = T0 and t = Tn is given by R(T0 , Tn ) =
δ−1 (
[P(T0 , Tn )]−1/n − 1
)
assuming P(Tn , Tn ) = 1, i.e., Tn is the maturity. Equation ( 5.1) is recovered
in the limit where n →∞ , δ → 0, nδ = ﬁxed (and equal to T − t in Eq. ( 5.1)).
© The Author(s) 2018
Z. Kakushadze and J. A. Serur, 151 T rading Strategies,
https://doi.org/10.1007/978-3-030-02792-6_5
99

100 Z. Kakushadze and J. A. Serur
makes n regular coupon payments at a ﬁxed uncompounded rate k at times
Ti = T0 + i δ, i = 1,2,..., n, and also pays $1 principal at maturity T .
The amount of each coupon payment is kδ,w h e r e δ is the payment period.
This income stream is equivalent to owning one T -bond plus kδ u n i t so fe a c h
Ti -bond, i = 1,..., n. The price of the coupon bond at time t then is
Pc(t, T ) = P(t, T ) + kδ
n∑
i =I (t )
P(t, Ti )( 5.2)
where I (t ) = min(i : t < Ti ).A tt i m e t = T0 we have
Pc(T0, T ) = P(T0, T ) + kδ
n∑
i =1
P(T0, Ti )( 5.3)
If we desire the coupon bond to start with its face value ( Pc(T0, T ) = 1), then
the corresponding coupon rate is given by
k = 1 − P(T0, T )
δ ∑ n
i =1 P(T0, Ti ) (5.4)
Floating Rate Bonds
A bond might also have ﬂoating coupon payments. Thus, consider a bond that
pays $1 at maturity T , and also makes coupon payments at times Ti = T0 +i δ,
i = 1,2,..., n, with amounts based on the variable rate (usually LIBOR—see
Sect. 5.15)
L(Ti −1) = 1
δ
[ 1
P(Ti −1, Ti ) − 1
]
(5.5)
The actual coupon payment at time Ti is
Xi = L(Ti −1)δ = 1
P(Ti −1, Ti ) − 1 (5.6)
which is the amount of interest we would get by buying $1’s worth of a Ti -
bond at time Ti −1.I n d e e d ,a Ti -bond is worth P(Ti −1, Ti ) at t = Ti −1,s o
$1’s worth a Ti -bond at t = Ti −1 is worth 1/P(Ti −1, Ti ) at t = Ti ,s ot h e

5 Fixed Income 101
interest earned is given by Eq. ( 5.6). The total value of the variable coupon
bond at t = T0 is given by:
V0 = 1 − [ P(T0, Tn ) − P(T0, T )] (5.7)
If T = Tn ,t h e nw eh a v e V0 = 1. This is because this bond is equivalent to the
following sequence of trades. At time t = T0 take $1 and buy T1-bonds with
it. At time t = T1 take the interest from the T1-bonds as the T1-coupon, and
buy T2-bonds with the leftover $1 principal. Repeat until we are left with $1
at time Tn . This has exactly the same cash ﬂow as the variable coupon bond, so
the initial prices must match. If T > Tn ,t h e n V0 < 1 and can be determined
as follows. First, note that
V0 = P(T0, T ) + V coupons
0 (5.8)
where V coupons
0 i st h et o t a lv a l u eo fa l l n coupon payments at t = T0.T h i s
value is independent of T and is determined from
P(T0, Tn ) + V coupons
0 = 1 (5.9)
which is the value of the variable coupon bond with maturity Tn .H e n c e
Eq. ( 5.7).
Swaps
Swaps are contracts that exchange a stream of ﬂoating rate payments for a
stream of ﬁxed rate payments or vice versa. A swap where we receive a stream of
ﬁxed rate payments in exchange for ﬂoating rate payments is simply a portfolio
which is long a ﬁxed coupon bond and short a variable coupon bond. The price
of the former at t = T0 is given by Eq. ( 5.3), while that of the latter is given by
Eq. ( 5.7). The ﬁxed rate that gives the swap initial null value is independent
of maturity T and given by
k = 1 − P(T0, Tn )
δ ∑ n
i =1 P(T0, Ti ) (5.10)

102 Z. Kakushadze and J. A. Serur
Duration and Convexity
Macaulay duration of a bond is a weighted average maturity of its cash ﬂows,
where the weights are the present values of said cash ﬂows. E.g., for a ﬁxed rate
coupon bond we have (see Eq. 5.3)
MacD(t, T ) = 1
Pc(t, T )
⎡
⎣(T − t ) P(t, T ) + kδ
n∑
i =I (t )
(Ti − t ) P(t, Ti )
⎤
⎦ (5.11)
Modiﬁed duration is deﬁned as (assuming parallel shifts in the yield curve) 2
ModD(t, T ) =− ∂ ln (Pc(t, T ))
∂R(t, T ) (5.12)
For continuous compounding, Macaulay duration and modiﬁed duration are
the same (see Eq. 5.1). For periodic compounding, they differ. For a constant
yield R(t,τ) = Y = const. (for all t <τ< T ), they are related via (see fn. 1):
ModD(t, T ) = MacD(t, T )/(1 + Y δ) ( 5.13)
Modiﬁed duration is a measure of the relative bond price sensitivity to changes
in the interest rates: /Delta1Pc(t, T )/Pc(t, T ) ≈− ModD(t, T )/Delta1R(t, T ) (for
parallel shifts /Delta1R(t,τ) = /Delta1R = const.,f o ra l l t <τ< T ). Similarly, dollar
duration deﬁned as
DD(t, T ) =− ∂Pc(t, T )
∂R(t, T ) = ModD(t, T ) Pc(t, T )( 5.14)
is a measure of the absolute bond price sensitivity to changes in the interest rates.
Convexity of a bond is deﬁned as (again, assuming parallel shifts) 3
C(t, T ) =− 1
Pc(t, T )
∂2 Pc(t, T )
∂R(t, T )2 (5.15)
2I.e., ∂R(t, τ)/∂R(t, T ) = 1 for all t <τ< T . For nonuniform shifts things get complicated.
3For some literature on various properties of bonds, see, e.g., Baxter and Rennie ( 1996), Bessembinder
and Maxwell ( 2008), ˇCerovi´ce ta l .( 2014), Chance and Jordan ( 1996), Chen et al. ( 2007), Chen et
al. ( 2010), Christensen ( 1999), Cole and Young ( 1995), Fabozzi ( 2006a, 2012a, b), Fabozzi and Mann
(2010), Henderson (2003), Horvath (1998), Hotchkiss and Ronen ( 2002), Hull (2012), Hull et al. ( 2005),
Jostova et al. ( 2013), Kakushadze ( 2015a), Leland and Panos ( 1997), Litterman and Scheinkman ( 1991),
Macaulay ( 1938), Martellini et al. ( 2003), Osborne ( 2005), Samuelson ( 1945), Stulz ( 2010), T uckman
and Serrat ( 2012).

5 Fixed Income 103
and corresponds to nonlinear effects in the response of the bond price to interest
rate changes:
/Delta1Pc(t, T )/Pc(t, T ) ≈− ModD(t, T )/Delta1R(t, T ) + 1
2 C(t, T ) [/Delta1R(t, T )]2 (5.16)
5.2 Strategy: Bullets
In a bullet portfolio, all bonds have the same maturity date T thereby targeting
a speciﬁc segment of the yield curve. The maturity can be picked based on the
trader’s outlook on the future interest rates: if the interest rates are expected
to fall (i.e., the bond prices to rise), then picking a longer maturity would
make more sense; if the interest rates are expected to rise (i.e., the bond prices
to fall), then a shorter maturity would be more warranted; however, if the
trader is uncertain about the future interest rates, a more diversiﬁed portfolio
(e.g., a barbell/ladder portfolio—see below) is in order (as opposed to a bullet
portfolio). T ypically, the bonds in a bullet portfolio are purchased over time,
which mitigates the interest rate risk to some extent: if the interest rates rise,
the later bond purchases will be at higher rates; if the interest rates fall, the
earlier bond purchases will have higher yields.
4
5.3 Strategy: Barbells
In this strategy, all purchased bonds are concentrated in two maturities T1
(short maturity) and T2 (long maturity), so this portfolio is a combination of
two bullet strategies. This strategy takes advantage of the higher yields from
the long-maturity bonds while hedging the interest rate risk with the short-
maturity bonds: if the interest rates rise, the long-maturity bonds will lose
value, but the proceeds from the short-maturity bonds can be reinvested at
higher rates. 5 The modiﬁed duration (call it D) of the barbell strategy is the
same as the modiﬁed duration (call it D∗) of a bullet strategy with a mid-range
maturity (call it T∗, T1 < T∗ < T2). However, the convexity (call it C)o ft h e
barbell strategy is higher than the convexity (call it C∗)o ft h i sb u l l e ts t r a t e g y .
4For some literature on bullet and barbell (see below) strategies, see, e.g., Fabbozzi et al. ( 2006), Grantier
(1988), Jones ( 1991), Mann and Ramanlal ( 1997), Pascalau and Poirier ( 2015), Su and Knowles ( 2010),
Wilner ( 1996), Yamada ( 1999).
5Flattening/steepening of the yield curve (the spread between the short-term and long-term interest rates
decreases/increases) has a positive/negative impact on the value of the portfolio.

104 Z. Kakushadze and J. A. Serur
Intuitively this can be understood by noting that modiﬁed duration scales
approximately linearly with maturity, while convexity scales approximately
quadratically with maturity. For illustrative purposes and simplicity, let us
consider a barbell strategy consisting of w1 dollars’ worth of zero-coupon
bonds with short maturity T1 and w2 dollars’ worth of zero-coupon bonds
with long maturity T2 (each bond has $1 face value). Furthermore, let us
assume continuous compounding and a constant yield Y .W et h e nh a v e
D = ˜w1 T1 + ˜w2 T2
˜w1 + ˜w2
(5.17)
T∗ = D∗ = D (5.18)
C = ˜w1 T 2
1 + ˜w2 T 2
2
˜w1 + ˜w2
(5.19)
C∗ = T 2
∗ (5.20)
where ˜w1 = w1 exp(−T1 Y ) and ˜w2 = w2 exp(−T2 Y ). Straightforward
algebra gives
C − C∗ = ˜w1 ˜w2
(˜w1 + ˜w2)2 (T2 − T1)2 > 0 (5.21)
Higher convexity of the barbell portfolio provides a better protection against
parallel shifts in the yield curve. However, this comes at the expense of a lower
overall yield.
5.4 Strategy: Ladders
A ladder is a bond portfolio with (roughly) equal capital allocations into bonds
of n different maturities Ti , i = 1,..., n (where the number of rungs n is
sizable, e.g., n = 10). The maturities are equidistant: Ti +1 = Ti + δ.T h i s
is a duration-targeting strategy, 6 which maintains an approximately constant
duration by selling shorter-maturity bonds as they approach maturity and
replacing them with new longer-maturity bonds. A ladder portfolio aims to
diversify the interest rate and reinvestment risks
7 by avoiding exposure to only
a few maturities (as in bullets and barbells). It also generates a regular revenue
6For some literature on ladder and duration-targeting strategies, see, e.g., Bierwag et al. ( 1978), Bohlin
and Strickland ( 2004), Cheung et al. ( 2010), Dyl and Martin ( 1986), Fridson and Xu ( 2014), Judd et al.
(2011), Langetieg et al. ( 1990), Leibowitz and Bova ( 2013), Leibowitz et al. ( 2014, 2015).
7The reinvestment risk is the risk that the proceeds (from coupon payments and/or principal) would be
reinvested at a lower rate than the original investment.

5 Fixed Income 105
stream from the coupons of each bond. The maturity of a ladder portfolio can
be deﬁned as the average maturity:
T = 1
n
n∑
i =1
Ti (5.22)
The income is higher for higher values of T ; however, so is the interest rate risk.
5.5 Strategy: Bond Immunization
Bond immunization is used in cases such as a predetermined future cash obli-
gation. A simple solution would be to purchase a zero-coupon bond with the
required maturity (and desirable/acceptable yield). However, such a bond may
not always be available in the market, so a portfolio of bonds with varying
maturities must be used instead. Such a portfolio is subject to the interest rate
and reinvestment risks. One way to mitigate these risks is to build a portfolio
whose duration matches the maturity of the future cash obligation (thereby
“immunizing” the bond portfolio against parallel shifts in the yield curve).
Consider a portfolio of bonds with 2 different maturities T
1, T2 and the cor-
responding durations D1, D2 (where “duration” means modiﬁed duration).
Let: the dollar amounts invested in these bonds be P1, P2; the total amount to
be invested be P; the desired duration of the portfolio be D (which is related
to the maturity T∗ of the future cash obligation—see below); and the constant
yield (which is assumed to be the same for all bonds —see below) be Y .T h e n P
is ﬁxed using Y and the amount of the future obligation F :
P = F/(1 + Y δ)T∗/δ (5.23)
where we are assuming periodic compounding and δ is the length of each
compounding period (e.g., 1 year). 8 Then we have:
P1 + P2 = P (5.24)
P1 D1 + P2 D2 = PD (5.25)
where
D = T∗/(1 + Y δ) ( 5.26)
8For the sake of simplicity, in Eq. ( 5.23) the number n = T∗/δ of compounding periods is assumed to
be a whole number. Extension to non-integer T∗/δ is straightforward.

106 Z. Kakushadze and J. A. Serur
With 3 bonds, we can also match the convexity:
P1 + P2 + P3 = P (5.27)
P1 D1 + P2 D2 + P3 D3 = PD (5.28)
P1 C1 + P2 C2 + P3 C3 = PC (5.29)
where C1,C2,C3 are the convexities of the 3 bonds and
C = T∗(T∗ + δ)/(1 + Y δ)2 (5.30)
In practice, the yield curve changes over time, which (among other things)
requires that the portfolio be periodically rebalanced. This introduces nontriv-
ial transaction costs, which must also be accounted for. Furthermore, the yields
are not the same for all bonds in the portfolio, which introduces additional
complexity into the problem.
9
5.6 Strategy: Dollar-Duration-Neutral Butterfly
This is a zero-cost combination of a long barbell portfolio (with short T1 and
long T3 maturities) and a short bullet portfolio (with a medium maturity T2,
where T1 < T2 < T3). Let: the dollar amounts invested in the 3 bonds be
P1, P2, P3; and the corresponding modiﬁed durations be D1, D2, D3.T h e n
zero cost (i.e., dollar-neutrality) and the dollar-duration-neutrality (the latter
protects the portfolio from parallel shifts in the yield curve) imply that
P
1 + P3 = P2 (5.31)
P1 D1 + P3 D3 = P2 D2 (5.32)
This ﬁxes P1, P3 via P2. While the portfolio is immune to parallel shifts in the
yield curve, it is not immune to changes in the slope or the curvature of the
yield curve.
10
9For some literature on bond immunization, including more sophisticated optimization techniques, see,
e.g., Albrecht ( 1985), Alexander and Resnick ( 1985), Bierwag ( 1979), Bodie et al. ( 1996), Boyle ( 1978),
Christensen and Fabozzi ( 1985), De La Peña et al. ( 2017), Fisher and Weil (1971), Fong and Vasicek (1983,
1984), Hürlimann ( 2002, 2012), Iturricastillo and De La Peña ( 2010), Khang ( 1983), Kocherlakota et al.
(1988, 1990), Montrucchio and Peccati ( 1991), Nawalkha and Chambers ( 1996), Reddington ( 1952),
Reitano ( 1996), Shiu ( 1987, 1988), Zheng et al. ( 2003).
10For some literature on various butterﬂy bond strategies, see, e.g., Bedendo et al. ( 2007), Brooks and
Moskowitz ( 2017), Christiansen and Lund ( 2005), Fontaine and Nolin ( 2017), Gibson and Pritsker
(2000), Grieves ( 1999), Heidari and Wu ( 2003), Martellini et al. ( 2002).

5 Fixed Income 107
5.7 Strategy: Fifty-Fifty Butterfly
This is a variation of the standard butterﬂy. In the above notations for the
dollar-duration-neutral butterﬂy, we have
P1 D1 = P3 D3 = 1
2 P2 D2 (5.33)
So, the ﬁfty-ﬁfty butterﬂy is still dollar-duration-neutral, but it is no longer
dollar-neutral (i.e., it is not a zero-cost strategy). Instead, dollar durations of
the wings are the same (hence the term “ﬁfty-ﬁfty”). As a result, the strategy is
(approximately) neutral to small steepening and ﬂattening of the yield curve, to
wit, if the interest rate spread change between the body and the short-maturity
wing is equal to the spread change between the long-maturity wing and the
body. That is why this strategy is a.k.a. “neutral curve butterﬂy” (whose cost
is non-dollar-neutrality).
5.8 Strategy: Regression-Weighted Butterfly
Empirically, short-term interest rates are sizably more volatile than long-term
interest rates. 11 Therefore, the interest rate spread change between the body
and the short-maturity wing (of the butterﬂy—see above) can be expected to
be greater by some factor—call it β—than the spread change between the long-
maturity wing and the body (so, typically β> 1). This factor can be obtained
from historical data via, e.g., running a regression of the spread change between
the body and the short-maturity wing over the spread change between the long-
maturity wing and the body. Then, instead of Eq. ( 5.33), we have the following
dollar-duration-neutrality and “curve-neutrality” conditions:
P
1 D1 + P3 D3 = P2 D2 (5.34)
P1 D1 = β P3 D3 (5.35)
11See, e.g., Edwards and Susmel ( 2003), Joslin and Konchitchki ( 2018), Mankiw and Summers ( 1984),
Shiller ( 1979), Sill ( 1996), T urnovsky (1989).

108 Z. Kakushadze and J. A. Serur
Strategy: Maturity-Weighted Butterfly
This is a variation of the regression-weighted butterﬂy, where instead of ﬁxing
β in Eq. ( 5.35) via a regression based on historical data, this coefﬁcient is based
on the 3 bond maturities:
β = T2 − T1
T3 − T2
(5.36)
5.9 Strategy: Low-Risk Factor
As in stocks, empirical evidence suggests that lower-risk bonds tend to outper-
form higher-risk bonds on the risk-adjusted basis (“low-risk anomaly”). 12 One
can deﬁne “riskiness” of a bond using different metrics, e.g., bond credit rating
and maturity. For instance, a long portfolio can be built (see, e.g., Houweling
and van Vundert 2017) by taking Investment Grade bonds with credit ratings
AAA through A −, and then taking the bottom decile by maturity. Similarly,
one can take High Yield bonds with credit ratings BB+ through B −,a n dt h e n
take the bottom decile by maturity.
5.10 Strategy: Value Factor
“Value” for bonds (see, e.g., Correia et al. 2012; Houweling and van Vundert
2017; L ’Hoir and Boulhabel 2010) is trickier to deﬁne than for stocks. One
way is to compare the observed credit spread 13 to a theoretical prediction
therefor. One way to estimate the latter is, e.g., via a linear cross-sectional
(across N bonds labeled by i = 1,..., N ) regression (Houweling and van
Vundert 2017):
S
i =
K∑
r =1
βr Iir + γ Ti + ϵi (5.37)
S∗
i = Si − ϵi (5.38)
Here: Si is the credit spread; Iir is a dummy variable ( Iir = 1 if the bond
labeled by i has credit rating r ; otherwise, Iir = 0) for bond credit rating r
12For some literature, see, e.g., De Carvalho et al. ( 2014), Derwall et al. ( 2009), Frazzini and Pedersen
(2014), Houweling and van Vundert ( 2017), Ilmanen ( 2011), Ilmanen et al. ( 2004), Kozhemiakin ( 2007),
Ng and Phelps ( 2015).
13Credit spread is the difference between the bond yield and the risk-free rate.

5 Fixed Income 109
(which labels K credit ratings present among the N bonds, which can be one
of the 21 credit ratings) 14; Ti are bond maturities; βr ,γ are the regression
coefﬁcients; ϵi are the regression residuals; and S∗
i is the ﬁtted (theoretical)
value of the credit spread. The N × K matrix Iir has no columns with all zeros
(so K can be less than 21). Note that by deﬁnition, since each bond has one
and only one credit rating, we have
K∑
r =1
Iir = 1 (5.39)
so the intercept is subsumed in Iir (which is why there is no separate regression
coefﬁcient for the intercept). Next, value is deﬁned as Vi = ln(Si /S∗
i ) or
Vi = ϵi /S∗
i = Si /S∗
i − 1, and the bonds in the top decile by Vi are selected
for the portfolio.
5.11 Strategy: Carry Factor
Carry is deﬁned as the return from the appreciation of the bond value as the
bond rolls down the yield curve (see, e.g., Beekhuizen et al. 2016; Koijen et
al. 2018)
15:
C(t, t + /Delta1t, T ) = P(t + /Delta1t, T ) − P(t, T )
P(t, T ) (5.40)
Here /Delta1t is the period over which carry is computed. A simpliﬁcation arises
if we assume that the entire term structure of the interest rates stays constant,
i.e., the yield R(t, T ) = f (T − t ) is a function of only T − t (i.e., time to
maturity). Then, at time t + /Delta1t the yield is R(t + /Delta1t, T ) = R(t, T − /Delta1t ).
So, we have 16
C(t,t + /Delta1t, T ) = P(t + /Delta1t, T )|R(t +/Delta1t,T ) − P(t, T )|R(t,T )
P(t, T )|R(t,T )
=
= R(t, T )/Delta1t + Cro ll (t,t + /Delta1t, T )( 5.41)
14These credit ratings are AAA, AA+, AA, AA −,A + ,A ,A −, BBB+, BBB, BBB −, BB+, BB, BB −,B + ,B ,
B−, CCC+, CCC, CCC −, CC, C.
15Here, for the sake of simplicity, we consider zero-coupon bonds. The end-result below is also valid for
coupon bonds.
16For ﬁnanced portfolios, R(t, T ) in the second line of Eq. ( 5.41) is replaced by R(t, T ) −rf, where rf is
the risk-free rate. However, this overall shift does not affect the actual holdings in the carry strategy below.

110 Z. Kakushadze and J. A. Serur
where (taking into account the deﬁnition of the modiﬁed duration, Eq. ( 5.12))
Cro ll (t,t + /Delta1t, T ) = P(t + /Delta1t, T )|R(t,T −/Delta1t ) − P(t + /Delta1t, T )|R(t,T )
P(t, T )|R(t,T )
≈
≈− ModD(t, T ) [ R(t, T − /Delta1t ) − R(t, T )] (5.42)
So, if the term structure of the interest rates is constant, then carry C(t, t +
/Delta1t, T ) receives two contributions: (i) R(t, T )/Delta1t from the bond yield; and
(ii) Cro ll (t,t + /Delta1t, T ) from the bond rolling down the yield curve. A zero-
cost strategy can be built, e.g., by buying bonds in the top decile by carry and
selling bonds in the bottom decile.
5.12 Strategy: Rolling Down the Yield Curve
The objective of this strategy is to capture the “roll-down” component
Cro ll (t,t + /Delta1t, T ) of bond yields. These returns are maximized in the steep-
est segments of the yield curve. Therefore, the trader can, e.g., buy long- or
medium-term bonds from such segments and hold them while they are “rolling
down the curve”.
17 The bonds must be sold as they approach maturity and the
proceeds can be used to buy new long/medium-term bonds from the steepest
segment of the yield curve at that time.
5.13 Strategy: Yield Curve Spread (Flatteners
and Steepeners)
This strategy consists of buying or selling the yield curve spread. 18 The yield
curve spread is deﬁned as the difference between the yields of two bonds of
the same issuer with different maturities. If the interest rates are expected to
fall, the yield curve is expected to steepen. If the interest rates are expected to
rise, the yield curve is expected to ﬂatten. The yield curve spread strategy can
be summarized via the following rule:
17For some literature on the “rolling down the yield curve” strategies, see, e.g., Ang et al. ( 1998), Bieri
and Chincarini ( 2004, 2005), Dyl and Joehnk ( 1981) ,G r i e v e se ta l .(1999), Grieves and Marcus ( 1992),
Osteryoung et al. ( 1981), Pantalone and Platt ( 1984), Pelaez ( 1997).
18For some literature on yield curve spread strategies, the yield curve dynamics and related topics, see, e.g.,
Bernadell et al. ( 2005), Boyd and Mercer ( 2010), Chua et al. ( 2006), Diebold and Li ( 2002), Diebold et
al. ( 2006), Dolan ( 1999), Evans and Marshall ( 2007), Füss and Nikitina ( 2011), Jones ( 1991), Kalev and
Inder ( 2006), Krishnamurthy ( 2002), Shiller and Modigliani ( 1979).

5 Fixed Income 111
Rule =
{
Flattener: Short spread if interest rates are expected to rise
Steepener: Buy spread if interest rates are expected to fall (5.43)
Shorting the spread amounts to selling shorter-maturity bonds (a.k.a. the front
leg) and buying longer-maturity bonds (a.k.a. the back leg). Buying the spread
is the opposite trade: buying the front leg and selling the back leg. If the
yield curve has parallel shifts, this strategy can generate losses. Matching dollar
durations of the front and back legs immunizes the portfolio to small parallel
shifts in the yield curve.
5.14 Strategy: CDS Basis Arbitrage
A credit default swap (CDS) is insurance against default on a bond. 19 The
CDS price, known as the CDS spread, is a periodic (e.g., annual) premium
per dollar of the insured debt. The CDS essentially makes the bond a risk-free
instrument. Therefore, the CDS spread should equal the bond yield spread,
i.e., the spread between the bond yield and the risk-free rate. The difference
between the CDS spread and the bond spread is known as the CDS basis:
CDS basis = CDS spread − bond spread (5.44)
Negative basis indicates that the bond spread is too high relative to the CDS
spread, i.e., the bond is relatively cheap. The CDS arbitrage trade then amounts
to buying the bond and insuring it with the CDS
20 thereby generating a risk-
free proﬁt. 21
5.15 Strategy: Swap-Spread Arbitrage
This dollar-neutral strategy consists of a long (short) position in an interest
rate swap (see Sect. 5.1) and a short (long) position in a T reasury bond (with
the constant yield YTre asu ry ) with the same maturity as the swap. A long
(short) swap involves receiving (making) ﬁxed rate rswap coupon payments in
19For some literature on CDS basis arbitrage and related topics, see, e.g., Bai and Collin-Dufresne ( 2013),
Choudhry ( 2004, 2006, 2007), De Wit ( 2006), Fontana ( 2010), Fontana and Scheicher ( 2016), Kim et
al. ( 2016, 2017), Nashikkar et al. ( 2011), Rajan et al. ( 2007), Wang ( 2014), Zhu ( 2006).
20Note that the CDS is equivalent to a synthetic short bond position.
21In the case of positive basis, theoretically one would enter into the opposite trade, i.e., selling the bond
and selling the CDS. However, in practice this would usually imply that the trader already owns the bond
and the CDS, i.e., this would amount to unwinding an existing position.

112 Z. Kakushadze and J. A. Serur
exchange for making (receiving) variable rate coupon payments at LIBOR (the
London Interbank Offer Rate) L(t ). The short (long) position in the T reasury
bond generates (is ﬁnanced at) the “repo rate” (the discount rate at which the
central bank repurchases government securities from commercial banks) r (t )
in a margin account. The per-dollar-invested rate C(t ) at which this strategy
generates P&L is given by
C(t ) =± [C1 − C2(t )] (5.45)
C1 = rswap − YTre asu ry (5.46)
C2(t ) = L(t ) − r (t )( 5.47)
where the plus (minus) sign corresponds to the long (short) swap strategy.
The long (short) swap strategy is proﬁtable if LIBOR falls (rises). So, this is a
LIBOR bet. 22
References
Albrecht, P . (1985). A Note on Immunization Under a General Stochastic Equilibrium
Model of the T erm Structure. Insurance: Mathematics and Economics , 4 (4), 239–
244.
Alexander, G. J., & Resnick, B. G. (1985). Using Linear and Goal Programming to
Immunize Bond Portfolios. Journal of Banking & Finance , 9 (1), 35–54.
Ang, S., Alles, L., & Allen, D. (1998). Riding the Yield Curve: An Analysis of Inter-
national Evidence. Journal of Fixed Income , 8(3), 57–74.
Asgharian, H., & Karlsson, S. (2008). An Empirical Analysis of Factors Driving the
Swap Spread. Journal of Fixed Income , 18(2), 41–56.
Aussenegg, W ., Götz, L., & Jelic, R. (2014). European Asset Swap Spreads and the
Credit Crisis. European Journal of Finance , 22(7), 572–600.
Bai, J., & Collin-Dufresne, P . (2013). The CDS-Bond Basis (Working Paper). Available
online: https://ssrn.com/abstract=2024531.
Baxter, M., & Rennie, A. (1996). Financial Calculus: An Introduction to Derivative
Pricing. Cambridge, UK: Cambridge University Press.
Bedendo, M., Cathcart, L., & El-Jahel, L. (2007). The Slope of the T erm Structure of
Credit Spreads: An Empirical Investigation. Journal of Financial Research , 30(2),
237–257.
22For some literature on swap spreads and related topics, see, e.g., Asgharian and Karlsson ( 2008),
Aussenegg et al. ( 2014), Chen and Selender ( 1994), Collin-Dufresne and Solnik ( 2001), Duarte et al.
(2006), Dubil ( 2011), Dufﬁe ( 1996), Dufﬁe and Singleton ( b), Feldhütter and Lando ( 2008), Fisher
(2002), Jermann ( 2016), Jordan and Jordan ( 1997), Kambhu ( 2006), Keane ( 1996), Klingler and Sun-
daresan ( 2016), Kobor et al. ( 2005), Lang et al. ( 1998), Liu et al. ( 2006), Minton ( 1997).

5 Fixed Income 113
Beekhuizen, P ., Duyvesteyn, J., Martens, M., & Zomerdijk, C. (2016). Carry
Investing on the Yield Curve (Working Paper). Available online: http://ssrn.com/
abstract=2808327.
Bernadell, C., Coche, J., & Nyholm, K. (2005). Yield Curve Prediction for the Strategic
Investor (Working Paper Series, No. 472). Frankfurt am Main, Germany: Euro-
pean Central Bank. Available online: https://www.ecb.europa.eu/pub/pdf/scpwps/
ecbwp472.pdf?1dc8846d9df4642959c54aa73cee81ad.
Bessembinder, H., & Maxwell, W . (2008). Markets: T ransparency and the Corporate
Bond Market. Journal of Economic Perspectives , 22(2), 217–234.
Bieri, D. S., & Chincarini, L. B. (2004). Riding the Yield Curve: Diversiﬁcation of
Strategies (Working Paper). Available online: https://ssrn.com/abstract=547682.
Bieri, D. S., & Chincarini, L. B. (2005). Riding the Yield Curve: A Variety of Strategies.
Journal of Fixed Income , 15 (2), 6–35.
Bierwag, G. O. (1979). Dynamic Portfolio Immunization Policies. Journal of Banking
& Finance , 3(1), 23–41.
Bierwag, G. O., & Kaufman, G. (1978). Bond Portfolio Strategy Simulations: A
Critique. Journal of Financial and Quantitative Analysis , 13(3), 519–525.
Bodie, Z., Kane, A., & Marcus, A. J. (1996). Investments.N e wY o r k ,N Y :M c G r a w -
Hill.
Bohlin, S., & Strickland, G. (2004). Climbing the Ladder: How to Manage Risk in
Your Bond Portfolio. American Association of Individual Investors Journal (July),
5–8.
Boyd, N. E., & Mercer, J. M. (2010). Gains from Active Bond Portfolio Management
Strategies. Journal of Fixed Income , 19 (4), 73–83.
Boyle, P . P . (1978). Immunization Under Stochastic Models of the T erm Structure.
Journal of the Institute of Actuaries , 105 (2), 177–187.
Brooks, J., & Moskowitz, T . J. (2017). Yield Curve Premia (Working Paper). Available
online: https://ssrn.com/abstract=2956411.
ˇCerovi´c, S., Pepi´ c, M., ˇCerovi´c, S., & ˇCerovi´c, N. (2014). Duration and Convexity
of Bonds. Singidunum Journal of Applied Sciences , 11(1), 53–66.
Chance, D. M., & Jordan, J. V . (1996). Duration, Convexity, and Time as Compo-
nents of Bond Returns. Journal of Fixed Income , 6 (2), 88–96.
Chen, A. H., & Selender, A. K. (1994). Determination of Swap Spreads: An Empirical
Analysis (Cox School of Business Historical Working Papers, No. 170). Dallas, TX:
Southern Methodist University. Available online: http://scholar.smu.edu/business_
workingpapers/170.
Chen, L., Lesmond, D. A., & Wei, J. (2007). Corporate Yield Spreads and Bond
Liquidity. Journal of Finance , 62(1), 119–149.
Chen, Z., Mao, C. X., & Wang, Y. (2010). Why Firms Issue Callable Bonds: Hedging
Investment Uncertainty. Journal of Corporate Finance , 16 (4), 588–607.
Cheung, C. S., Kwan, C. C. Y., & Sarkar, S. (2010). Bond Portfolio Laddering: A
Mean-Variance Perspective. Journal of Applied Finance , 20(1), 103–109.

114 Z. Kakushadze and J. A. Serur
Choudhry, M. (2004). The Credit Default Swap Basis: Analysing the Relationship
Between Cash and Synthetic Credit Markets. Journal of Derivatives Use, T rading
and Regulation , 10(1), 8–26.
Choudhry, M. (2006). Revisiting the Credit Default Swap Basis: Further Analysis of
the Cash and Synthetic Credit Market Differential. Journal of Structured Finance ,
11(4), 21–32.
Choudhry, M. (2007). T rading the CDS Basis: Illustrating Positive and Negative Basis
Arbitrage T rades. Journal of T rading, 2(1), 79–94.
Christensen, M. (1999). Duration and Convexity for Bond Portfolios. Finanzmarkt
und Portfolio Management , 13(1), 66–72.
Christensen, P . E., & Fabozzi, F . J. (1985). Bond Immunization: An Asset Liability
Optimization Strategy. In F . J. Fabozzi & I. M. Pollack (Eds.), The Handbook of
Fixed Income Securities (2nd ed., pp. 676–703). Homewood, IL: Dow Jones-Irwin.
Christiansen, C., & Lund, J. (2005). Revisiting the Shape of the Yield Curve: The
Effect of Interest Rate Volatility (Working Paper). Available online: https://ssrn.com/
abstract=264139.
Chua, C. T ., Koh, W . T . H., & Ramaswamy, K. (2006). Proﬁting from Mean-Reverting
Yield Curve T rading Strategies. Journal of Fixed Income , 15 (4), 20–33.
Cole, C. S., & Young, P . J. (1995). Modiﬁed Duration and Convexity with Semiannual
Compounding. Journal of Economics and Finance , 19 (1), 1–15.
Collin-Dufresne, P ., & Solnik, B. (2001). On the T erm Structure of Default Premia
in the Swap and LIBOR Markets. Journal of Finance , 56 (3), 1095–1115.
Correia, M. M., Richardson, S. A., & T una, A. I. (2012). Value Investing in Credit
Markets. Review of Accounting Studies , 17 (3), 572–609.
De Carvalho, R. L., Dugnolle, P ., Lu, X., & Moulin, P . (2014). Low-Risk Anomalies
in Global Fixed Income: Evidence from Major Broad Markets. Journal of Fixed
Income, 23(4), 51–70.
De La Peña, J. I., Garayeta, A., & Iturricastillo, I. (2017). Dynamic Immunisation
Does Not Imply Cash Flow Matching: A Hard Application to Spain. Economic
Research—Ekonomska Istraživanja , 30(1), 238–255.
De Wit, J. (2006). Exploring the CDS-Bond Basis (Working Paper). Available online:
https://ssrn.com/abstract=1687659.
Derwall, J., Huij, J., & De Zwart, G. B. (2009). The Short-T erm Corporate Bond
Anomaly (Working Paper). Available online: https://ssrn.com/abstract=1101070.
Diebold, F . X., & Li, C. (2002). Forecasting the T erm Structure of Government Bond
Yields. Journal of Econometrics , 130(2), 337–364.
Diebold, F . X., Rudebusch, G. D., & Aruoba, S. B. (2006). The Macroeconomy
and the Yield Curve: A Dynamic Latent Factor Approach. Journal of Econometrics ,
131(1–2), 309–338.
Dolan, C. P . (1999). Forecasting the Yield Curve Shape. Journal of Fixed Income , 9 (1),
92–99.
Duarte, J., Longstaff, F . A., & Yu, F . (2006). Risk and Return in Fixed-Income Arbi-
trage: Nickels in Front of a Steamroller? Review of Financial Studies , 20(3), 769–
811.

5 Fixed Income 115
Dubil, R. (2011). Hedge Funds: Alpha, Beta and Replication Strategies. Journal of
Financial Planning , 24 (10), 68–77.
Dufﬁe, D. (1996). Special Repo Rates. Journal of Finance , 51(2), 493–526.
Dufﬁe, D., & Singleton, K. J. (1997). An Econometric Model of the T erm Structure
of Interest Rate Swap Yields. Journal of Finance , 52(4), 1287–1321.
Dyl, E. A., & Joehnk, M. D. (1981). Riding the Yield Curve: Does It Work? Journal
of Portfolio Management , 7 (3), 13–17.
Dyl, E. A., & Martin, S. A. (1986). Another Look at Barbells V ersus Ladders. Journal
of Portfolio Management , 12(3), 54–59.
Edwards, S., & Susmel, R. (2003). Interest-Rate V olatility in Emerging Markets.
Review of Economics and Statistics , 85 (2), 328–348.
Evans, C. L., & Marshall, D. A. (2007). Economic Determinants of the Nominal
T reasury Yield Curve. Journal of Monetary Economics , 54 (7), 1986–2003.
Fabozzi, F . J. (2006). Fixed Income Mathematics: Analytical & Statistical T echniques .
New York, NY: McGraw-Hill.
Fabozzi, F . J. (2012a). Bond Markets, Analysis, and Strategies . Upper Saddle River, NJ:
Prentice Hall.
Fabozzi, F . J. (2012b). Institutional Investment Management: Equity and Bond Portfolio
Strategies and Applications . Hoboken, NJ: Wiley.
Fabozzi, F . J., & Mann, S. V . (2010). Introduction to Fixed Income Analytics: Relative
Value Analysis, Risk Measures, and Valuation . Hoboken, NJ: Wiley.
Fabozzi, F . J., Martellini, L., & Priaulet, P . (2006). Advanced Bond Portfolio Manage-
ment. Best Practices in Modeling and Strategies . Hoboken, NJ: Wiley.
Feldhütter, P ., & Lando, D. (2008). Decomposing Swap Spreads. Journal of Financial
Economics, 88(2), 375–405.
Fisher, M. (2002). Special Repo Rates: An Introduction. Federal Reserve Bank of
Atlanta, Economic Review , 87 (2), 27–43.
Fisher, L., & Weil, R. L. (1971). Coping with the Risk of Interest-Rate Fluctuations:
Returns to Bondholders from Naïve and Optimal Strategies. Journal of Business ,
44 (4), 408–431.
Fong, H. G., & Vasicek, O. A. (1983). The T radeoff Between Return and Risk in
Immunized Portfolios. Financial Analysts Journal , 39 (5), 73–78.
Fong, H. G., & Vasicek, O. A. (1984). A Risk Minimizing Strategy for Portfolio
Immunization. Journal of Finance , 39 (5), 1541–1546.
Fontaine, J.-F ., & Nolin, G. (2017). Measuring Limits of Arbitrage in Fixed-Income
Markets (Staff Working Paper, No. 2017-44). Ottawa, Canada: Bank of Canada.
Fontana, A. (2010). The Persistent Negative CDS-Bond Basis During the 2007/08
Financial Crisis (Working Paper). Available online: http://www.unive.it/media/
allegato/DIP/Economia/Working_papers/Working_papers_2010/WP_DSE_
fontana_13_10.pdf .
Fontana, A., & Scheicher, M. (2016). An Analysis of Euro Area Sovereign CDS and
Their Relation with Government Bonds. Journal of Banking & Finance , 62, 126–
140.

116 Z. Kakushadze and J. A. Serur
Frazzini, A., & Pedersen, L. H. (2014). Betting Against Beta. Journal of Financial
Economics, 111(1), 1–25.
Fridson, M. S., & Xu, X. (2014). Duration T argeting: No Magic for High-Yield
Investors. Financial Analysts Journal , 70(3), 28–33.
Füss, R., & Nikitina, O. (2011). Explaining Yield Curve Dynamics. Journal of Fixed
Income, 21(2), 68–87.
Gibson, M. S., & Pritsker, M. (2000). Improving Grid-Based Methods for Estimating
Value at Risk of Fixed-Income Portfolios. Journal of Risk , 3(2), 65–89.
Grantier, B. J. (1988). Convexity and Bond Performance: The Benter the Better.
Financial Analysts Journal , 44 (6), 79–81.
Grieves, R. (1999). Butterﬂy T rades. Journal of Portfolio Management , 26 (1), 87–95.
Grieves, R., Mann, S. V ., Marcus, A. J., & Ramanlal, P . (1999). Riding the Bill Curve.
Journal of Portfolio Management , 25 (3), 74–82.
Grieves, R., & Marcus, A. J. (1992). Riding the Yield Curve: Reprise. Journal of
Portfolio Management, 18(4), 67–76.
Heidari, M., & Wu, L. (2003). Are Interest Rate Derivatives Spanned by the T erm
Structure of Interest Rates? Journal of Fixed Income , 13(1), 75–86.
Henderson, T . M. (2003). Fixed Income Strategy: The Practitioner’s Guide to Riding the
Curve. Chichester, UK: Wiley.
Horvath, P . A. (1998). A Measurement of the Errors in Intra-Period Compounding
and Bond Valuation: A Short Extension. Financial Review , 23(3), 359–363.
Hotchkiss, E. S., & Ronen, R. (2002). The Informational Efﬁciency of the Corporate
Bond Market: An Intraday Analysis. Review of Financial Studies, 15 (5), 1325–1354.
Houweling, P ., & van Vundert, J. (2017). Factor Investing in the Corporate Bond
Market. Financial Analysts Journal , 73(2), 100–115.
Hull, J. C. (2012). Options, Futures and Other Derivatives . Upper Saddle River, NJ:
Prentice Hall.
Hull, J., Predescu, M., & White, A. (2005). Bond Prices, Default Probabilities and
Risk Premiums. Journal of Credit Risk , 1(2), 53–60.
Hürlimann, W . (2002). On Immunization, Stop-Loss Order and the Maximum Shiu
Measure. Insurance: Mathematics and Economics , 31(3), 315–325.
Hürlimann, W . (2012). On Directional Immunization and Exact Matching. Commu-
nications in Mathematical Finance , 1(1), 1–12.
Ilmanen, A. (2011). Expected Returns: An Investor’s Guide to Harvesting Market Rewards.
Hoboken, NJ: Wiley.
Ilmanen, A., Byrne, R., Gunasekera, H., & Minikin, R. (2004). Which Risks Have
Been Best Rewarded? Journal of Portfolio Management , 30(2), 53–57.
Iturricastillo, I., & De La Peña, J. I. (2010). Absolute Immunization Risk as General
Measure of Immunization Risk. Análisis Financiero , 114 (3), 42–59.
Jermann, U. J. (2016). Negative Swap Spreads and Limited Arbitrage (Working Paper).
Available online: https://ssrn.com/abstract=2737408.
Jones, F . J. (1991). Yield Curve Strategies. Journal of Fixed Income , 1(2), 43–48.
Jordan, B. D., & Jordan, S. (1997). Special Repo Rates: An Empirical Analysis. Journal
of Finance , 52(5), 2051–2072.

5 Fixed Income 117
Joslin, S., & Konchitchki, Y. (2018). Interest Rate V olatility, the Yield Curve, and the
Macroeconomy. Journal of Financial Economics , 128(2), 344–362.
Jostova, G., Nikolova, S., Philipov, A., & Stahel, C. W . (2013). Momentum in Cor-
porate Bond Returns. Review of Financial Studies , 26 (7), 1649–1693.
Judd, K. L., Kubler, F ., & Schmedders, K. (2011). Bond Ladders and Optimal Port-
folios. Review of Financial Studies , 24 (12), 4123–4166.
Kakushadze, Z. (2015). Phynance. Universal Journal of Physics and Application , 9 (2),
64–133. Available online: https://ssrn.com/abstract=2433826.
Kalev, P . S., & Inder, B. A. (2006). The Information Content of the T erm Structure
of Interest Rates. Applied Economics , 38(1), 33–45.
Kambhu, J. (2006). T rading Risk, Market Liquidity, and Convergence T rading in
the Interest Rate Swap Spread. Federal Reserve Bank of New York, Economic Policy
Review, 12(1), 1–13.
Keane, F . (1996). Repo Rate Patterns for New T reasury Notes. Federal Reserve Bank of
New York, Current Issues in Economics and Finance , 2(10), 1–6.
Khang, C. H. (1983). A Dynamic Global Portfolio Immunization Strategy in the
World of Multiple Interest Rate Changes: A Dynamic Immunization and Minimax
Theorem. Journal of Financial and Quantitative Analysis , 18(3), 355–363.
Kim, G. H., Li, H., & Zhang, W . (2016). CDS-Bond Basis and Bond Return Pre-
dictability. Journal of Empirical Finance , 38, 307–337.
Kim, G. H., Li, H., & Zhang, W . (2017). The CDS-Bond Basis Arbitrage and the
Cross Section of Corporate Bond Returns. Journal of Futures Markets , 37 (8), 836–
861.
Klingler, S., & Sundaresan, S. M. (2016). An Explanation of Negative Swap Spreads:
Demand for Duration from Underfunded Pension Plans (Working Paper). Available
online: https://ssrn.com/abstract=2814975.
Kobor, A., Shi, L., & Zelenko, I. (2005). What Determines U.S. Swap Spreads? (World
Bank Working Paper No. 62). Washington, DC: World Bank.
Kocherlakota, R., Rosenbloom, E., & Shiu, E. (1988). Algorithms for Cash-Flow
Matching. T ransactions of Society of Actuaries , 40, 477–484.
Kocherlakota, R., Rosenbloom, E., & Shiu, E. (1990). Cash-Flow Matching and
Linear Programming Duality. T ransactions of Society of Actuaries , 42, 281–293.
Koijen, R. S. J., Moskowitz, T . J., Pedersen, L. H., & Vrugt, E. B. (2018). Carry.
Journal of Financial Economics , 127 (2), 197–225.
Kozhemiakin, A. V . (2007). The Risk Premium of Corporate Bonds. Journal of Portfolio
Management, 33(2), 101–109.
Krishnamurthy, A. (2002). The Bond/Old-Bond Spread. Journal of Financial Eco-
nomics, 66 (2), 463–506.
Langetieg, T . C., Leibowitz, L., & Kogelman, S. (1990). Duration T argeting and the
Management of Multiperiod Returns. Financial Analysts Journal , 46 (5), 35–45.
Lang, L. H. P ., Litzenberger, R. H., & Liu, A. L. (1998). Determinants of Interest
Rate Swap Spreads. Journal of Banking & Finance , 22(12), 1507–1532.
Leibowitz, M. L., & Bova, A. (2013). Duration T argeting and Index Convergence.
Morgan Stanley Investment Management Journal , 3(1), 73–80.

118 Z. Kakushadze and J. A. Serur
Leibowitz, M. L., Bova, A., & Kogelman, S. (2014). Long-T erm Bond Returns Under
Duration T argeting. Financial Analysts Journal , 70(1), 31–51.
Leibowitz, M. L., Bova, A., & Kogelman, S. (2015). Bond Ladders and Rolling Yield
Convergence. Financial Analysts Journal , 71(2), 32–46.
Leland, E. C., & Panos, N. (1997). The Puttable Bond Market: Structure, Historical
Experience, and Strategies. Journal of Fixed Income , 7 (3), 47–60.
L ’Hoir, M., & Boulhabel, M. (2010). A Bond-Picking Model for Corporate Bond
Allocation. Journal of Portfolio Management , 36 (3), 131–139.
Litterman, R. B., & Scheinkman, J. (1991). Common Factors Affecting Bond Returns.
Journal of Fixed Income , 1(1), 54–61.
Liu, J., Longstaff, F . A., & Mandell, R. E. (2006). The Market Price of Risk in Interest
Rate Swaps: The Roles of Default and Liquidity Risks. Journal of Business , 79 (5),
2337–2360.
Macaulay, F . R. (1938).Some Theoretical Problems Suggested by the Movements of Interest
Rates, Bond Yields and Stock Prices in the United States Since 1856 .N e wY o r k ,N Y :
NBER Inc.
Mankiw, N. G., & Summers, L. H. (1984). Do Long-T erm Interest Rates Overreact
to Short-T erm Interest Rates? Brookings Papers on Economic Activity , 1, 223–242.
Mann, S. V ., & Ramanlal, P . (1997). The Relative Performance of Yield Curve Strate-
gies. Journal of Portfolio Management , 23(4), 64–70.
Martellini, L., Priaulet, P ., & Priaulet, S. (2002). Understanding the Butterﬂy Strategy.
Journal of Bond T rading and Management , 1(1), 9–19.
Martellini, L., Priaulet, P ., & Priaulet, S. (2003). Fixed Income Securities: Valuation,
Risk Management and Portfolio Strategies . Hoboken, NJ: Wiley.
Minton, B. A. (1997). An Empirical Examination of Basic Valuation Models for Plain
Vanilla U.S. Interest Rate Swaps. Journal of Financial Economics , 44 (2), 251–277.
Montrucchio, L., & Peccati, L. (1991). A Note on Shiu-Fisher-Weil Immunization
Theorem. Insurance: Mathematics and Economics , 10(2), 125–131.
Nashikkar, A., Subrahmanyam, M. G., & Mahanti, S. (2011). Liquidity and Arbitrage
in the Market for Credit Risk. Journal of Financial and Quantitative Analysis , 46 (3),
627–656.
Nawalkha, S. K., & Chambers, D. R. (1996). An Improved Immunization Strategy:
M-absolute. Financial Analysts Journal , 52(5), 69–76.
Ng, K. Y., & Phelps, B. D. (2015). The Hunt for a Low-Risk Anomaly in the USD
Corporate Bond Market. Journal of Portfolio Management , 42(1), 63–84.
Osborne, M. J. (2005). On the Computation of a Formula for the Duration of a Bond
that Yields Precise Results. Quarterly Review of Economics and Finance , 45 (1), 161–
183.
Osteryoung, J. S., McCarty, D. E., & Roberts, G. S. (1981). Riding the Yield Curve
with T reasury Bills. Financial Review , 16 (3), 57–66.
Pantalone, C., & Platt, H. (1984). Riding the Yield Curve. Journal of Financial Edu-
cation, 13, 5–9.
Pascalau, R., & Poirier, R. (2015). Bootstrapping the Relative Performance of Yield
Curve Strategies. Journal of Investment Strategies , 4 (2), 55–81.

5 Fixed Income 119
Pelaez, R. F . (1997). Riding the Yield Curve: T erm Premiums and Excess Returns.
Review of Financial Economics , 6 (1), 113–119.
Rajan, A., McDermott, G., & Roy, R. (Eds.). (2007). The Structured Credit Handbook .
Hoboken, NJ: Wiley.
Reddington, F . M. (1952). Review of the Principles of Life Insurance Valuations.
Journal of the Institute of Actuaries , 78(3), 286–340.
Reitano, R. (1996). Non-parallel Yield Curve Shifts and Stochastic Immunization.
Journal of Portfolio Management , 22(2), 71–78.
Samuelson, P . A. (1945). The Effect of Interest Rate Increases on the Banking System.
American Economic Review , 35 (1), 16–27.
Shiller, R. J. (1979). The V olatility of Long-T erm Interest Rates and Expectations
Models of the T erm Structure. Journal of Political Economy , 87 (6), 1190–1219.
Shiller, R. J., & Modigliani, F . (1979). Coupon and T ax Effects on New and Seasoned
Bond Yields and the Measurement of the Cost of Debt Capital. Journal of Financial
Economics, 7 (3), 297–318.
Shiu, E. S. W . (1987). On the Fisher-Weil Immunization Theorem. Insurance: Math-
ematics and Economics , 6 (4), 259–266.
Shiu, E. S. W . (1988). Immunization of Multiple Liabilities. Insurance: Mathematics
and Economics , 7 (4), 219–224.
Sill, K. (1996). The Cyclical V olatility of Interest Rates. Business Review of the Federal
Reserve Bank of Philadelphia (January/February), 15–29.
Stulz, R. M. (2010). Credit Default Swaps and the Credit Crisis. Journal of Economic
Perspectives, 24 (1), 73–92.
Su, E., & Knowles, T . W . (2010). Measuring Bond Portfolio Value at Risk and Expected
Shortfall in US T reasury Market. Asia Paciﬁc Management Review , 15 (4), 477–501.
T uckman, B., & Serrat, A. (2012). Fixed Income Securities: T ools for T oday ’s Markets
(3rd ed.). Hoboken, NJ: Wiley.
T urnovsky, S. J. (1989). The T erm Structure of Interest Rates and the Effects of
Macroeconomic Policy. Journal of Money, Credit and Banking , 21(3), 321–347.
Wang, L. (2014). Margin-Based Asset Pricing and the Determinants of the CDS Basis.
Journal of Fixed Income , 24 (2), 61–78.
Wilner, R. (1996). A New T ool for Portfolio Managers: Level, Slope, and Curvature
Durations. Journal of Fixed Income , 6 (1), 48–59.
Yamada, S. (1999). Risk Premiums in the JGB Market and Application to Investment
Strategies. Journal of Fixed Income , 9 (2), 20–41.
Zheng, H.,Thomas, L. C., & Allen, D. E. (2003).The Duration Derby: A Comparison
of Duration Strategies in Asset Liability Management. Journal of Bond T rading and
Management, 1(4), 371–380.
Zhu, H. (2006). An Empirical Comparison of Credit Spreads between the Bond
Market and the Credit Default Swap Market. Journal of Financial Services Research ,
29 (3), 211–235.
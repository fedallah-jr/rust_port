# Chapter 14: Miscellaneous Assets

14
Miscellaneous Assets
14.1 Strategy: Inflation Hedging—Inflation
Swaps
This strategy amounts to buying (selling) inﬂation swaps in order to exchange
a ﬁxed (ﬂoating) rate of inﬂation for a ﬂoating (ﬁxed) rate. Inﬂation swaps con-
ceptually are similar to interest rate swaps (see section “ Swaps”i nC h a p t e r5).
A buyer (seller) of an inﬂation swap is long (short) the inﬂation and receives the
ﬂoating (ﬁxed) rate. The buyer has a positive return if the inﬂation exceeds the
expected inﬂation (i.e., the swap ﬁxed rate, a.k.a. the “breakeven rate”). The
ﬁxed rate typically is calculated as the interest rate spread between the T reasury
notes/bonds (as applicable) and T reasury Inﬂation-Protected Securities (TIPS)
with the same maturity as that of the swap. The ﬂoating rate usually is based
on an inﬂation index such as the Consumer Price Index (CPI). The most com-
mon type of inﬂation swap is the zero-coupon inﬂation swap (ZC), which has
only one cash ﬂow at maturity T (measured in years). This cash ﬂow is the
difference between the ﬁxed rate cash ﬂow C
fix e d and the ﬂoating rate cash
ﬂow C fl o a t i n g . These cash ﬂows, per $1 notational, are given by:
C fix e d = (1 + K )T − 1 (14.1)
C fl o a t i n g = I (T )/I (0) − 1 (14.2)
Here: K is the ﬁxed rate; and I (t ) is the CPI value at time t (t = 0 is the time
at which the swap contract is entered into). Another type of inﬂation swaps
is the year-on-year inﬂation swap (YoY), which references annual inﬂation
(as opposed to the cumulative inﬂation referenced by the zero-coupon swap).
© The Author(s) 2018
Z. Kakushadze and J. A. Serur, 151 T rading Strategies,
https://doi.org/10.1007/978-3-030-02792-6_14
205

206 Z. Kakushadze and J. A. Serur
Thus, assuming for simplicity annual payments, we have (here t = 1,..., T
is measured in years) 1:
C fix e d (t ) = K (14.3)
C fl o a t i n g (t ) = I (t )/I (t − 1) − 1 (14.4)
14.2 Strategy: TIPS-Treasury Arbitrage
This strategy is based on the empirical observation that T reasury bonds tend
to be overvalued relative to TIPS 2 almost all the time (see, e.g., Campbell et
al. 2009; Driessen et al. 2017; Fleckenstein 2012;H a u b r i c he ta l .2012). The
strategy amounts to selling a T reasury bond (whose price is PTre asu ry ,ﬁ x e d
coupon rate is rTre asu ry , and maturity is T ) and offsetting this short position
with a synthetic portfolio, which precisely replicates the T reasury bond coupon
and principal payments, but costs less than the T reasury bond. This synthetic
portfolio is constructed by buying TIPS (whose price is PTIP S and maturity
T is the same as that of the T reasury bond) with a ﬁxed coupon rate r and n
coupon payments at times ti , i = 1,..., n (with tn = T ), and simultaneously
selling n zero-coupon inﬂation swaps with maturities ti , the ﬁxed rate K ,a n d
the notionals Ni = r +δti ,T per $1 of the TIPS principal. The cash ﬂows (per
$1 notional) at t = ti are given by (as above, I (t ) is the CPI value at time t ;
1For some literature on inﬂation swaps and related topics, see, e.g., Belgrade and Benhamou ( 2004),
Belgrade et al. ( 2004), Bouzoubaa and Osseiran ( 2010), Christensen et al. ( 2010), Deacon et al. ( 2004),
Fleming and Sporn ( 2013), Haubrich et al. ( 2012), Hinnerich (2008), Jarrow and Yildirim ( 2003), Kenyon
(2008), Lioui and Poncet ( 2005), Martellini et al. ( 2015), Mercurio ( 2005), Mercurio and Moreni ( 2006,
2009), Mercurio and Yildirim ( 2008).
2TIPS pay semiannual ﬁxed coupons at a ﬁxed rate, but the coupon payments (and principal) are adjusted
based on inﬂation. For some literature on TIPS, inﬂation-indexed products and related topics, see, e.g.,
Adrian and Wu ( 2010), Ang et al. ( 2008), Bardong and Lehnert ( 2004), Barnes et al. ( 2010), Barr and
Campbell (1997), Bekaert and Wang ( 2010), Buraschi and Jiltsov ( 2005), Campbell et al. ( 2017), Chen et
al. ( 2010), Chernov and Mueller ( 2012), Christensen and Gillan ( 2012), D’Amico et al. ( 2018), Deacon
et al. ( 2004), Dudley et al. ( 2009), Evans ( 1998), Fleckenstein et al. ( 2017), Fleming and Krishnan
(2012), Grishchenko and Huang ( 2013), Grishchenko et al. ( 2016), Gürkaynak et al. ( 2010), Hördahl
and T ristani (2012), Hördahl and T ristani ( 2014), Hunter and Simon ( 2005), Jacoby and Shiller ( 2008),
Joyce et al. ( 2010), Kandel et al. ( 1996), Kitsul and Wright ( 2013), Kozicki and Tinsley ( 2012), Mehra
(2002), Pennacchi ( 1991), Pﬂueger and Viceira ( 2011), Remolona et al. ( 1998), Roll ( 1996, 2004), Sack
and Elsasser ( 2004), Seppälä ( 2004), Shen ( 2006), Shen and Corning ( 2001), Woodward ( 1990), Yared
and V eronesi (1999).

14 Miscellaneous Assets 207
also, time is measured in the units of the (typically, semiannual) compounding
periods):
CTIP S (ti ) = Ni I (ti )/I (0)( 14.5)
Cswap (ti ) = Ni
[
(1 + K )ti − I (ti )/I (0)
]
(14.6)
Ctotal (ti ) = Cswap (ti ) + CTIP S (ti ) = Ni (1 + K )ti (14.7)
So, the synthetic portfolio converts the indexed payments from TIPS into ﬁxed
payments with the effective coupon rates ref f (ti ) = r (1 + K )ti . These syn-
thetic coupon payments almost replicate the T reasury bond coupons rTre asu ry .
The exact matching involves small long or short positions in STRIPS, 3 which
are given by (see, e.g., Fleckenstein et al. ( 2013) for details)
S(ti ) = D(ti )
{[
rTre asu ry − ref f (ti )
]
+ δti ,T
[
1 − (1 + K )ti
]}
(14.8)
where D(τ) is the value of the STRIPS with maturity τ at time t = 0
(i.e., D(τ) is a discount factor). In Eq. ( 14.8) the second term in the curly
brackets (which is proportional to δti ,T and is nonzero only for i = n, i.e.,
at maturity T ) is included as we must also match the principals at maturity.
Note that the STRIPS positions are established at t = 0. The net cash ﬂow
C(0) at t = 0 is given by (note that the net cash ﬂows at t > 0 are all null by
replication)
C(0) = PTre asu ry − PTIP S −
n∑
i =1
S(ti )( 14.9)
Empirically C(0) tends to be positive (even after transaction costs). Hence
arbitrage.
14.3 Strategy: Weather Risk—Demand Hedging
Various businesses and sectors of the economy can be affected by weather condi-
tions, both directly and indirectly. Weather risk is hedged using weather deriva-
tives. There are no “tradable” weather indexes, so various synthetic indexes have
been created. The most common ones are based on temperature. The cooling-
3STRIPS = “Separate T rading of Registered Interest and Principal of Securities”. Essentially, STRIPS are
zero-coupon discount bonds.

208 Z. Kakushadze and J. A. Serur
degree-days (CDD) and heating-degree-days (HDD) measure extreme high
temperatures and extreme low temperatures, respectively 4:
ICDD =
n∑
i =1
max(0, Ti − Tbase )( 14.10)
IHDD =
n∑
i =1
max(0, Tbase − Ti )( 14.11)
Ti = T min
i + T max
i
2 (14.12)
Here: i = 1,..., n labels days; n is the life of the contract (a week, a month or
a season) measured in days; T min
i and T max
i are the minimum and maximum
temperatures recorded on the day labeled by i ;a n d Tbase = 65◦ F. Then, the
demand risk for heating days can, e.g., be hedged by a short futures position
or a long put option position with the hedge ratios given by (here (Cov) Var
is serial (co)variance):
h
HDD
fu t u r e s = Cov(qw, IHDD )/Var(IHDD )( 14.13)
h HDD
put =− Cov (qw,max(K − IHDD ,0))/
Var(max(K − IHDD , 0)) (14.14)
Here: qw is the portion of the demand affected by weather conditions (as there
might be other, exogenous, non-weather-related components to the demand);
and K is the strike price. Similarly, the demand risk for cooling days can, e.g.,
4For some literature on weather derivatives, weather indexes and related topics, see, e.g., Alaton et al.
(2010), Barrieu and El Karoui ( 2002), Barrieu and Scaillet ( 2010), Benth ( 2003), Benth and Saltyte-
Benth ( 2005, 2007), Benth et al. ( 2007), Bloesch and Gourio ( 2015), Brocket et al. ( 2005, 2010), Brody
et al. ( 2002), Campbell and Diebold ( 2005), Cao and Wei ( 2000, 2004), Cartea and Figueroa ( 2005),
Chaumont et al. ( 2006), Chen et al. ( 2006), Corbally and Dang ( 2002), Davis ( 2001), Dischel ( 1998a,
b, 1999), Dorﬂeitner and Wimmer ( 2010), Dornier and Queruel ( 2000), Ederington ( 1979), Geman
(1998), Geman and Leonardi ( 2005), Ghiulnara and Viegas ( 2010), Golden et al. ( 2007), Göncü ( 2012),
Hamisultane ( 2009), Hanley ( 1999), Härdle and López Cabrera ( 2011), Huang et al. ( 2008), Huault
and Rainelli-Weis ( 2011), Hunter ( 1999), Jain and Baile ( 2000), Jewson ( 2004a, b), Jewson et al. ( 2005),
Jewson and Caballero ( 2003), Lazo et al. ( 2011), Lee and Oren ( 2009), Leggio and Lien ( 2002), Mraoua
(2007), Müller and Grandi ( 2000), Oetomo and Stevenson ( 2005), Parnaudeau and Bertrand ( 2018),
Perez-Gonzalez and Yun ( 2010), Richards et al. ( 2004), Saltyte-Benth and Benth ( 2012), Schiller et al.
(2010), Svec and Stevenson ( 2007), Swishchuk and Cui ( 2013), T ang and Jang ( 2011), Thornes ( 2006),
V edenov and Barnett ( 2004), Wilson ( 2016), Woodard and Garcia ( 2008), Yang et al. ( 2009), Zapranis
and Alexandridis ( 2008), Zapranis and Alexandridis ( 2009), Zeng ( 2000).

14 Miscellaneous Assets 209
be hedged by a long futures position or a long call option position with the
hedge ratios given by:
hCDD
fu t u r e s = Cov(qw, ICDD )/Var(ICDD )( 14.15)
hCDD
call = Cov (qw, max(ICDD − K , 0))/
Var(max(ICDD − K , 0)) (14.16)
14.4 Strategy: Energy—Spark Spread
The spark spread is the difference between the wholesale price of electricity
and the price of natural gas required to produce it. 5 A spark spread can be built
by, e.g., taking a short position in electricity futures and a long position in the
corresponding number of fuel futures. Such positions are used by electricity
producers to hedge against changes in the electricity price or in the cost of
fuel, as well as by traders or speculators who want to make a bet on a power
plant. The number of fuel futures is determined by the so-called heat rate
H , which measures the efﬁciency with which the plant converts fuel into
electricity:
H = Q
F /Q E (14.17)
Here: Q F is the amount of fuel used to produce the amount of electricity Q E ;
Q F is measured in MMBtu; Btu = British thermal unit, which is approximately
1055 Joules; MBtu = 1000 Btu; MMBtu = 1,000,000 Btu; Q E is measured
in Mwh = Megawatt hour; the heat rate H is measured in MMBtu/Mwh.
The spark spread is measured in $/Mwh. So, if the price of electricity is PE
(measured in $/Mwh) and the price of fuel is PF (measured in $/MMBtu),
then the spark spread is given by
S = PE − HP F (14.18)
5So, the spark spread measures a gross margin of a gas-ﬁred power plant excluding all other costs for
operation, maintenance, capital, etc. Also, if the power plant uses fuel other than natural gas, then the
corresponding spread has a different name. For coal it is called “dark spread”; for nuclear power it is called
“quark spread”; etc. For some literature on energy spreads, energy hedging, and related topics, see, e.g.,
Benth and Kettler ( 2010), Benth et al. ( 2014), Carmona and Durrleman ( 2003), Cassano and Sick ( 2013),
Deng et al. ( 2001), Edwards ( 2009), Elias et al. ( 2016), Emery and Liu ( 2002), Fiorenzani ( 2006), Fusaro
and James ( 2005), Hsu ( 1998), James ( 2003), Kaminski ( 2004), Li and Kleindorfer ( 2009), Maribu et al.
(2007), Martínez and T orró ( 2018), Wang and Min ( 2013).

210 Z. Kakushadze and J. A. Serur
The hedge ratio for the futures is affected by the available futures contract
sizes. Thus, an electricity futures contract is FE = 736 Mwh, and a gas futures
contract is FF =10,000 MMBtu. So, the hedge ratio is given by
h = HF E /FF (14.19)
which generally is not a whole number. Therefore, it is (approximately, within
the desired precision) represented as a ratio h ≈ NF /NE with the lowest
possible denominator NE ,w h e r e NF and NE are whole numbers. Then the
hedge consists of buying NF gas futures contracts for every NE sold electricity
futures contracts.
References
Adrian, T ., & Wu, H. (2010). The T erm Structure of Inﬂation Expectations (Federal
Reserve Bank of New York Staff Reports, No. 362). Available online: https://www.
newyorkfed.org/medialibrary/media/research/staff_reports/sr362.pdf .
Alaton, P ., Djehiche, B., & Stillberger, D. (2010). On Modelling and Pricing Weather
Derivatives. Applied Mathematical Finance , 9 (1), 1–20.
Ang, A., Bekaert, G., & Wei, M. (2008). The T erm Structure of Real Rates and
Expected Inﬂation. Journal of Finance , 63(2), 797–849.
Bardong, F ., & Lehnert, T . (2004). TIPS, Break-Even Inﬂation, and Inﬂation Forecasts.
Journal of Fixed Income , 14 (3), 15–35.
Barnes, M. L., Bodie, Z., T riest, R. K., & Wang, J. C. (2010). A TIPS Scorecard: Are
They Accomplishing Their Objectives? Financial Analysts Journal , 66 (5), 68–84.
Barr, D. G., & Campbell, J. Y. (1997). Inﬂation, Real Interest Rates, and the Bond
Market: A Study of UK Nominal and Index-Linked Government Bond Prices.
Journal of Monetary Economics , 39 (3), 361–383.
Barrieu, P ., & El Karoui, N. (2002). Optimal Design of Weather Derivatives. ALGO
Research, 5 (1), 79–92.
Barrieu, P ., & Scaillet, O. (2010). A Primer on Weather Derivatives. In J. A. Filar
& A. Haurie (Eds.), Uncertainty and Environmental Decision Making: A Hand-
book of Research and Best Practice. International Series in Operations Research &
Management Science (V ol. 138). New York, NY: Springer U.S.
Bekaert, G., & Wang, X. (2010). Inﬂation Risk and the Inﬂation Risk Premium.
Economic Policy, 25 (64), 755–806.
Belgrade, N., & Benhamou, E. (2004). Reconciling Year on Year and Zero Coupon Inﬂa-
tion Swap: A Market Model Approach (Working Paper). Available online: https://
ssrn.com/abstract=583641.
Belgrade, N., Benhamou, E., & Koehler, E. (2004). A Market Model for Inﬂation
(Working Paper). Available online: https://ssrn.com/abstract=576081.

14 Miscellaneous Assets 211
Benth, F . E. (2003). On Arbitrage-Free Pricing of Weather Derivatives Based on
Fractional Brownian Motion. Applied Mathematical Finance , 10 (4), 303–324.
Benth, F . E., & Kettler, P . C. (2010). Dynamic Copula Models for the Spark Spread.
Quantitative Finance , 11(3), 407–421.
Benth, F . E., Kholodnyi, V . A., & Laurence, P . (Eds.). (2014). Quantitative Energy
Finance: Modeling, Pricing, and Hedging in Energy and Commodity Markets .N e w
York, NY: Springer.
Benth, F . E., & Saltyte-Benth, J. (2005). Stochastic Modelling of T emperature Vari-
ations with a View T owards Weather Derivatives. Applied Mathematical Finance ,
12(1), 53–85.
Benth, F . E., & Saltyte-Benth, J. (2007). The V olatility of T emperature and Pricing
of Weather Derivatives. Quantitative Finance , 7 (5), 553–561.
Benth, F . E., Saltyte-Benth, J., & Koekebakker, S. (2007). Putting a Price on T emper-
ature. Scandinavian Journal of Statistics , 34 (4), 746–767.
Bloesch, J., & Gourio, F . (2015). The Effect of Winter Weather on U.S. Economic
Activity. Federal Reserve Bank of Chicago, Economic Perspectives , 39 (1), 1–20.
Bouzoubaa, M., & Osseiran, A. (2010). Exotic Options and Hybrids: A Guide to Struc-
turing, Pricing and T rading . Chichester, UK: Wiley.
Brockett, P ., Golden, L. L., Wen, M., & Yang, C. (2010). Pricing Weather Derivatives
Using the Indifference Pricing Approach. North American Actuarial Journal , 13(3),
303–315.
Brockett, P . L., Wang, M., & Yang, C. (2005). Weather Derivatives and Weather Risk
Management. Risk Management and Insurance Review , 8(1), 127–140.
Brody, D., Syroka, J., & Zervos, M. (2002). Dynamical Pricing of Weather Derivatives.
Quantitative Finance , 2(3), 189–198.
Buraschi, A., & Jiltsov, A. (2005). Inﬂation Risk Premia and the Expectations Hypoth-
esis. Journal of Financial Economics , 75 (2), 429–490.
Campbell, S. D., & Diebold, F . X. (2005). Weather Forecasting for Weather Deriva-
tives. Journal of the American Statistical Association , 100 (469), 6–16.
Campbell, J. Y., Shiller, R. J., & Viceira, L. M. (2009). Understanding Inﬂation-
Indexed Bond Markets. In D. Romer & J. Wolfers (Eds.), Brookings Papers on
Economic Activity (pp. 79–120). Washington, DC: Brookings Institution Press.
Campbell, J. Y., Sunderam, A., & Viceira, L. M. (2017). Inﬂation Bets or Deﬂation
Hedges? The Changing Risks of Nominal Bonds. Critical Finance Review , 6 (2),
263–301.
Cao, M., & Wei, J. (2000). Pricing the Weather. Risk (May), 67–70.
Cao, M., & Wei, J. (2004). Weather Derivatives Valuation and Market Price of
Weather Risk. Journal of Futures Markets , 24 (11), 1065–1089.
Carmona, R., & Durrleman, V . (2003). Pricing and Hedging Spread Options. SIAM
Review, 45 (4), 627–685.
Cartea, A., & Figueroa, M. (2005). Pricing in Electricity Markets: A Mean Reverting
Jump Diffusion Model with Seasonality. Applied Mathematical Finance , 12(4),
313–335.

212 Z. Kakushadze and J. A. Serur
Cassano, M., & Sick, G. (2013). Valuation of a Spark Spread: An LM6000 Power
Plant. European Journal of Finance , 18(7–8), 689–714.
Chaumont, S., Imkeller, P ., & Müller, M. (2006). Equilibrium T rading of Climate
and Weather Risk and Numerical Simulation in a Markovian Framework. Stochastic
Environment Research and Risk Assessment , 20 (3), 184–205.
Chen, R.-R., Liu, B., & Cheng, X. (2010). Pricing the T erm Structure of Inﬂation
Risk Premia: Theory and Evidence from TIPS. Journal of Empirical Finance , 17 (4),
702–721.
Chen, G., Roberts, M. C., & Thraen, C. S. (2006). Managing Dairy Proﬁt Risk
Using Weather Derivatives. Journal of Agricultural and Resource Economics , 31(3),
653–666.
Chernov, M., & Mueller, P . (2012). The T erm Structure of Inﬂation Expectations.
Journal of Financial Economics , 106 (2), 367–394.
Christensen, J. H. E., & Gillan, J. M. (2012). Could the U.S. T reasury Beneﬁt from Issu-
ing More TIPS? (Federal Reserve Bank of San Francisco, Working Papers Series, No.
2011-16). Available online: https://www.frbsf.org/economic-research/ﬁles/wp11-
16bk.pdf .
Christensen, J. H. E., Lopez, J. A., & Rudebusch, G. D. (2010). Inﬂation Expectations
and Risk Premiums in an Arbitrage-Free Model of Nominal and Real Bond Yields.
Journal of Money, Credit, and Banking , 42(6), 143–178.
Corbally, M., & Dang, P . (2002). Underlying Markets and Indexes. In E. Banks
(Ed.), Weather Risk Management: Market, Products and Applications .L o n d o n ,U K :
Palgrave Macmillan.
D’Amico, S., Kim, D., & Wei, M. (2018). Tips from TIPS: The Informational Content
of T reasury Inﬂation-Protected Security Prices.Journal of Financial and Quantitative
Analysis, 53(1), 395–436.
Davis, M. (2001). Pricing Weather Derivatives by Marginal Value. Quantitative
Finance, 1(3), 305–308.
Deacon, M., Derry, A., & Mirfendereski, D. (2004). Inﬂation-Indexed Securities:
Bonds, Swaps and Other Derivatives . Chichester, UK: Wiley.
Deng, S.-J., Johnson, B., & Sogomonian, A. (2001). Exotic Electricity Options and
the Valuation of Electricity Generation and T ransmission Assets. Decision Support
Systems, 30 (3), 383–392.
Dischel, B. (1998a). At Last: A Model for Weather Risk. Energy and Power Risk
Management, 11(3), 20–21.
Dischel, B. (1998b). Black-Scholes Won’t Do. Energy and Power Risk Management ,
11(10), 8–9.
Dischel, B. (1999). Shaping History for Weather Risk Management. Energy and Power
Risk Management , 12(8), 13–15.
Dorﬂeitner, G., & Wimmer, M. (2010). The Pricing of T emperature Futures at the
Chicago Mercantile Exchange. Journal of Banking & Finance , 34 (6), 1360–1370.
Dornier, F ., & Queruel, M. (2000). Caution to the Wind. Energy and Power Risk
Management,
13(8), 30–32.

14 Miscellaneous Assets 213
Driessen, J., Nijman, T ., & Simon, Z. (2017). The Missing Piece of the Puzzle: Liquidity
Premiums in Inﬂation-Indexed Markets (Working Paper). Available online: https://
ssrn.com/abstract=3042506.
Dudley, W ., Roush, J. E., & Steinberg, M. (2009). The Case for Tips: An Examination
of the Costs and Beneﬁts. Federal Reserve Bank of New York, Economic Policy Review ,
15 (1), 1–17.
Ederington, L. H. (1979). The Hedging Performance of the New Futures Markets.
Journal of Finance , 34 (1), 157–170.
Edwards, D. W . (2009). Energy T rading & Investing: T rading, Risk Management and
Structuring Deals in the Energy Market .N e wY o r k ,N Y :M c G r a w - H i l l .
Elias, R. S., Wahab, M. I. M., & Fang, L. (2016). The Spark Spread and Clean Spark
Spread Option Based Valuation of a Power Plant with Multiple T urbines. Energy
Economics, 59, 314–327.
Emery, G. W ., & Liu, Q. (2002). An Analysis of the Relationship Between Electricity
and Natural Gas Futures Prices. Journal of Futures Markets , 22(2), 95–122.
Evans, M. D. D. (1998). Real Rates, Expected Inﬂation, and Inﬂation Risk Premia.
Journal of Finance , 53(1), 187–218.
Fiorenzani, S. (2006). Quantitative Methods for Electricity T rading and Risk Manage-
ment: Advanced Mathematical and Statistical Methods for Energy Finance .L o n d o n ,
UK: Palgrave Macmillan.
Fleckenstein, M. (2012). The Inﬂation-Indexed Bond Puzzle (Working Paper). Avail-
able online: https://ssrn.com/abstract=2180251.
Fleckenstein, M., Longstaff, F . A., & Lustig, H. N. (2013). Why Does the T reasury
Issue TIPS? The TIPS-T reasury Bond Puzzle.Journal of Finance, 69 (5), 2151–2197.
Fleckenstein, M., Longstaff, F . A., & Lustig, H. N. (2017). Deﬂation Risk. Review of
Financial Studies , 30 (8), 2719–2760.
Fleming, M. J., & Krishnan, N. (2012). The Microstructure of the TIPS Market.
Federal Reserve Bank of New York, Economic Policy Review , 18(1), 27–45.
Fleming, M. J., & Sporn, J. R. (2013). T rading Activity and Price T ransparency in the
Inﬂation Swap Market. Federal Reserve Bank of New York, Economic Policy Review ,
19 (1), 45–58.
Fusaro, P . C., & James, T . (2005).Energy Hedging in Asia: Market Structure and T rading
Opportunities. London, UK: Palgrave Macmillan.
Geman, H. (1998). Insurance and Weather Derivatives: From Exotic Options to Exotic
Underlyings. London, UK: Risk Books.
Geman, H., & Leonardi, M.-P . (2005). Alternative Approaches to Weather Derivatives
Pricing. Managerial Finance , 31(6), 46–72.
Ghiulnara, A., & Viegas, C. (2010). Introduction of Weather-Derivative Concepts:
Perspectives for Portugal. Journal of Risk Finance , 11(1), 9–19.
Golden, L. L., Wang, M., & Yang, C. (2007). Handling Weather Related Risks through
the Financial Markets: Considerations of Credit Risk, Basis Risk, and Hedging.
Journal of Risk and Insurance , 74 (2), 319–346.
Göncü, A. (2012). Pricing T emperature-Based Weather Derivatives in China. Journal
of Risk Finance , 13(1), 32–44.

214 Z. Kakushadze and J. A. Serur
Grishchenko, O. V ., & Huang, J.-Z. (2013). Inﬂation Risk Premium: Evidence from
the TIPS Market. Journal of Fixed Income , 22(4), 5–30.
Grishchenko, O. V ., Vanden, J. M., & Zhang, J. (2016). The Informational Content
of the Embedded Deﬂation Option in TIPS. Journal of Banking & Finance , 65,
1–26.
Gürkaynak, R. S., Sack, B., & Wright, J. H. (2010). The TIPS Yield Curve and
Inﬂation Compensation. American Economic Journal: Macroeconomics , 2(1), 70–
92.
Hamisultane, H. (2009). Utility-Based Pricing of Weather Derivatives. European Jour-
nal of Finance , 16 (6), 503–525.
Hanley, M. (1999). Hedging the Force of Nature. Risk Professional , 5 (4), 21–25.
Härdle, W . K., & López Cabrera, B. (2011). The Implied Market Price of Weather
Risk. Applied Mathematical Finance , 19 (1), 59–95.
Haubrich, J., Pennacchi, G., & Ritchken, P . (2012). Inﬂation Expectations, Real Rates,
and Risk Premia: Evidence from Inﬂation Swaps. Review of Financial Studies , 25 (5),
1588–1629.
Hinnerich, M. (2008). Inﬂation-Indexed Swaps and Swaptions. Journal of Banking &
Finance, 32(11), 2293–2306.
Hördahl, P ., & T ristani, O. (2012). Inﬂation Risk Premia in the T erm Structure of
Interest Rates. Journal of the European Economic Association , 10 (3), 634–657.
Hördahl, P ., & T ristani, O. (2014). Inﬂation Risk Premia in the Euro Area and the
United States. International Journal of Central Banking , 10 (3), 1–47.
Hsu, M. (1998). Spark Spread Options Are Hot! Electricity Journal , 11(2), 28–39.
Huang, H., Shiu, Y., & Lin, P . (2008). HDD and CDD Option Pricing with Market
Price of Weather Risk for T aiwan. Journal of Futures Markets , 28(8), 790–814.
Huault, I., & Rainelli-Weis, H. (2011). A Market for Weather Risk? Conﬂicting
Metrics, Attempts at Compromise, and Limits to Commensuration. Organization
Studies, 32(10), 1395–1419.
Hunter, R. (1999). Managing Mother Nature. Derivatives Strategy , 4 (2), 15–19.
Hunter, D. M., & Simon, D. P . (2005). Are TIPS the “Real” Deal? A Conditional
Assessment of Their Role in a Nominal Portfolio. Journal of Banking & Finance ,
29 (2), 347–368.
Jacoby, G., & Shiller, I. (2008). Duration and Pricing of TIPS. Journal of Fixed Income ,
18(2), 71–84.
Jain, G., & Baile, C. (2000). Managing Weather Risks. Strategic Risk (September),
28–31.
James, T . (2003). Energy Price Risk: T rading and Price Risk Management .L o n d o n ,U K :
Palgrave Macmillan.
Jarrow, R., & Yildirim, Y. (2003). Pricing T reasury Inﬂation Protected Securities and
Related Derivatives Using an HJM Model. Journal of Financial and Quantitative
Analysis, 38(2), 409–430.
Jewson, S. (2004a). Weather Derivative Pricing and the Distributions of StandardWeather
Indices on US T emperatures (Working Paper). Available online: https://ssrn.com/
abstract=535982.

14 Miscellaneous Assets 215
Jewson, S. (2004b). Introduction to Weather Derivative Pricing (Working Paper). Avail-
able online: https://ssrn.com/abstract=557831.
Jewson, S., Brix, A., & Ziehmann, C. (2005). Weather Derivative Valuation: The Mete-
orological, Statistical, Financial and Mathematical Foundations .C a m b r i d g e ,U K :
Cambridge University Press.
Jewson, S., & Caballero, R. (2003). Seasonality in the Statistics of Surface Air T em-
perature and the Pricing of Weather Derivatives. Meteorological Applications, 10 (4),
367–376.
Joyce, M., Lildholdt, P ., & Sorensen, S. (2010). Extracting Inﬂation Expectations and
Inﬂation Risk Premia from the T erm Structure: A Joint Model of the UK Nominal
and Real Yield Curves. Journal of Banking & Finance , 34 (2), 281–294.
Kaminski, V . (2004). Managing Energy Price Risk: The New Challenges and Solutions .
London, UK: Risk Books.
Kandel, S., Ofer, A. R., & Sarig, O. (1996). Real Interest Rates and Inﬂation: An
Ex-ante Empirical Analysis. Journal of Finance , 51(1), 205–225.
Kenyon, C. (2008). Inﬂation is Normal. Risk (July), 76–82.
Kitsul, Y., & Wright, J. H. (2013). The Economics of Options-Implied Inﬂation
Probability Density Functions. Journal of Financial Economics , 110 (3), 696–711.
Kozicki, S., & Tinsley, P . A. (2012). Effective Use of Survey Information in Estimating
the Evolution of Expected Inﬂation. Journal of Money, Credit and Banking , 44 (1),
145–169.
Lazo, J. K., Lawson, M., Larsen, P . H., & Waldman, D. M. (2011). U.S. Economic
Sensitivity to Weather Variability. Bulletin of the American Meteorological Society ,
92(6), 709–720.
Lee, Y., & Oren, S. (2009). An Equilibrium Pricing Model for Weather Derivatives
in a Multi-commodity Setting. Energy Economics , 31(5), 702–713.
Leggio, K., & Lien, D. (2002). Hedging Gas Bills with Weather Derivatives. Journal
of Economics and Finance , 26 (1), 88–100.
Li, L., & Kleindorfer, P . R. (2009). On Hedging Spark Spread Options in Electricity
Markets. Risk and Decision Analysis , 1(4), 211–220.
Lioui, A., & Poncet, P . (2005). General Equilibrium Pricing of CPI Derivatives. Journal
of Banking & Finance , 29 (5), 1265–1294.
Maribu, K. M., Galli, A., & Armstrong, M. (2007). Valuation of Spark-Spread Options
with Mean Reversion and Stochastic V olatility. International Journal of Electronic
Business Management , 5 (3), 173–181.
Martellini, L., Milhau, V ., & T arelli, A. (2015). Hedging Inﬂation-Linked Liabili-
ties Without Inﬂation-Linked Instruments Through Long/Short Investments in
Nominal Bonds. Journal of Fixed Income , 24 (3), 5–29.
Martínez, B., & T orró, H. (2018). Hedging Spark Spread Risk with Futures. Energy
Policy, 113, 731–746.
Mehra, Y. P . (2002). Survey Measures of Expected Inﬂation: Revisiting the Issues of
Predictive Content and Rationality. Federal Reserve Bank of Richmond, Economic
Quarterly, 88(3), 17–36.

216 Z. Kakushadze and J. A. Serur
Mercurio, F . (2005). Pricing Inﬂation-Indexed Derivatives.Quantitative Finance, 5 (3),
289–302.
Mercurio, F ., & Moreni, N. (2006). Inﬂation with a Smile. Risk, 19 (3), 70–75.
Mercurio, F ., & Moreni, N. (2009). Inﬂation Modelling with SABR Dynamics. Risk
(June), 106–111.
Mercurio, F ., & Yildirim, Y. (2008). Modelling Inﬂation. In B. Benaben & S. Gold-
enberg (Eds.), Inﬂation Risks and Products: The Complete Guide .L o n d o n ,U K :R i s k
Books.
Mraoua, M. (2007). T emperature Stochastic Modelling and Weather Derivatives Pric-
ing: Empirical Study with Moroccan Data. Afrika Statistika , 2(1), 22–43.
Müller, A., & Grandi, M. (2000). Weather Derivatives: A Risk Management T ool for
Weather-Sensitive Industries. Geneva Papers on Risk and Insurance , 25 (2), 273–287.
Oetomo, T ., & Stevenson, M. (2005). Hot or Cold? A Comparison of Different
Approaches to the Pricing of Weather Derivatives. Journal of Emerging Market
Finance, 4 (2), 101–133.
Parnaudeau, M., & Bertrand, J.-L. (2018). The Contribution of Weather Variability
to Economic Sectors. Applied Economics , 50 (43), 4632–4649.
Pennacchi, G. G. (1991). Identifying the Dynamics of Real Interest Rates and Inﬂa-
tion: Evidence Using Survey Data. Review of Financial Studies , 4 (1), 53–86.
Perez-Gonzalez, F ., & Yun, H. (2010). Risk Management and Firm Value: Evidence
from Weather Derivatives (Working Paper). Available online: https://ssrn.com/
abstract=1357385.
Pﬂueger, C. E., & Viceira, L. M. (2011). Inﬂation-Indexed Bonds and the Expectations
Hypothesis. Annual Review of Financial Economics , 3, 139–158.
Remolona, E. M., Wickens, M. R., & Gong, F . F . (1998). What W as the Market’s View
of UK Monetary Policy? Estimating Inﬂation Risk and Expected Inﬂation with Indexed
Bonds (Federal Reserve Bank of New York Staff Reports, No. 57). Available online:
https://ssrn.com/abstract=937350.
Richards, T ., Manfredo, M., & Sanders, D. (2004). Pricing Weather Derivatives.
American Journal of Agricultural Economics , 86 (4), 1005–1017.
Roll, R. (1996). U.S. T reasury Inﬂation-Indexed Bonds: The Design of a New Security.
Journal of Fixed Income , 6 (3), 9–28.
Roll, R. (2004). Empirical TIPS. Financial Analysts Journal , 60 (1), 31–53.
Sack, B., & Elsasser, R. (2004). T reasury Inﬂation-Indexed Debt: A Review of the
U.S. Experience. Federal Reserve Bank of New York, Economic Policy Review , 10 (1),
47–63.
Saltyte-Benth, J., & Benth, F . E. (2012). A Critical View on T emperature Modelling for
Application in Weather Derivatives Markets. Energy Economics , 34 (2), 592–602.
Schiller, F ., Seidler, G., & Wimmer, M. (2010). T emperature Models for Pricing
Weather Derivatives. Quantitative Finance ,
12(3), 489–500.
Seppälä, J. (2004). The T erm Structure of Real Interest Rates: Theory and Evidence
from UK Index-Linked Bonds. Journal of Monetary Economics , 51(7), 1509–1549.
Shen, P . (2006). Liquidity Risk Premia and Breakeven Inﬂation Rates. Federal Reserve
Bank of Kansas City, Economic Review , 91(2), 29–54.

14 Miscellaneous Assets 217
Shen, P ., & Corning, J. (2001). Can TIPS Help Identify Long-T erm Inﬂation Expec-
tations? Federal Reserve Bank of Kansas City, Economic Review , 86 (4), 61–87.
Svec, J., & Stevenson, M. (2007). Modelling and Forecasting T emperature Based
Weather Derivatives. Global Finance Journal , 18(2), 185–204.
Swishchuk, A., & Cui, K. (2013). Weather Derivatives with Applications to Canadian
Data. Journal of Mathematical Finance , 3(1), 81–95.
T ang, C. H., & Jang, S. H. (2011). Weather Risk Management in Ski Resorts: Finan-
cial Hedging and Geographical Diversiﬁcation. International Journal of Hospitality
Management, 30 (2), 301–311.
Thornes, J. E. (2006). An Introduction to Weather and Climate Derivatives. Weather,
58(5), 193–196.
V edenov, D. V ., & Barnett, B. J. (2004). Efﬁciency of Weather Derivatives as Primary
Crop Insurance Instruments. Journal of Agricultural Economics , 29 (3), 387–403.
Wang, C.-H., & Min, K. J. (2013). Electric Power Plant Valuation Based on Day-
Ahead Spark Spreads. Engineering Economist , 58(3), 157–178.
Wilson, D. J. (2016). The Impact of Weather on Local Employment: Using Big Data
on Small Places (Federal Reserve Bank of San Francisco Working Papers Series,
No. 2016-21). Available online: https://www.frbsf.org/economic-research/ﬁles/
wp2016-21.pdf .
Woodard, J., & Garcia, P . (2008). Weather Derivatives, Spatial Aggregation, and
Systemic Risk: Implications for Reinsurance Hedging. Journal of Agricultural and
Resource Economics , 33(1), 34–51.
Woodward, G. T . (1990). The Real Thing: A Dynamic Proﬁle of the T erm Structure
of Real Interest Rates and Inﬂation. Journal of Business , 63(3), 373–398.
Yang, C. C., Brockett, P . L., & Wen, M.-M. (2009). Basis Risk and Hedging Efﬁciency
of Weather Derivatives. Journal of Risk Finance , 10 (5), 517–536.
Yared, F ., & V eronesi, P . (1999).Short and Long Horizon T erm and Inﬂation Risk Premia
in the US T erm Structure: Evidence from an Integrated Model for Nominal and Real
Bond Prices Under Regime Shifts (Working Paper). Available online: https://ssrn.
com/abstract=199448.
Zapranis, A., & Alexandridis, A. (2008). Modelling the T emperature Time-Dependent
Speed of Mean Reversion in the Context of Weather Derivatives Pricing. Applied
Mathematical Finance , 15 (3–4), 355–386.
Zapranis, A., & Alexandridis, A. (2009). Weather Derivatives Pricing: Modeling the
Seasonal Residual Variance of an Ornstein-Uhlenbeck T emperature Process with
Neural Networks. Neurocomputing, 73(1–3), 37–48.
Zeng, L. (2000). Pricing Weather Derivatives. Journal of Risk Finance , 1(3), 72–78.
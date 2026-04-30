# Chapter 11: Structured Assets

11
Structured Assets
11.1 Generalities: Collateralized Debt
Obligations (CDOs)
A CDO is an asset-backed security (ABS) consisting of a basket of assets such
as bonds, credit default swaps, etc. It is divided into multiple tranches, which
consist of assets with different credit ratings and interest rates. Each tranche
has an attachment point a and a detachment point d. E.g., a 3–8% tranche
(for which a = 3% and d = 8%) means that it begins to lose value when
the underlying portfolio loss exceeds 3%; and when the underlying portfolio
loss exceeds 8%, the tranche value is completely wiped out. 1 A buyer (long
position) of a CDO tranche is a protection seller: in return for receiving periodic
premium payments, in the event of a default, the buyer has the obligation to
cover the default up to the size of the tranche. A seller (short position) of a
CDO tranche is a protection buyer: in return for making periodic premium
payments, the seller receives a payment in the event of a default. Synthetic
CDOs are “synthesized” through credit derivatives such as CDS (credit default
swaps—see Sect. 5.14) on a pool of reference entities (e.g., bonds, loans, names
of companies or countries). Reference pools for exchange-traded single-tranche
CDOs are CDS indexes such as CDX and iT raxx.
2
1Examples of tranches are (in the decreasing order of default risk and periodic premium payment rate):
equity 0–3% tranche; junior mezzanine 3–7% tranche; senior mezzanine 7–10% tranche; senior 10–15%
tranche; and super senior 15–30% tranche.
2For some literature on CDOs and related topics, see, e.g., Altman et al. ( 2005), Amato and Gyntelberg
(2005), Amato and Remolona ( 2003), Andersen and Sidenius ( 2005), Andersen et al. ( 2003), Belkin et al.
(1998), Bielecki et al. ( 2011), Bol et al. ( 2009), Boscher and Ward ( 2002), Cousin and Laurent ( 2012),
Das ( 2005), Davis and Lo ( 2001), Ding and Sherris ( 2011), Douglas ( 2007), Dufﬁe ( 2004), Dufﬁe and
© The Author(s) 2018
Z. Kakushadze and J. A. Serur, 151 T rading Strategies,
https://doi.org/10.1007/978-3-030-02792-6_11
181

182 Z. Kakushadze and J. A. Serur
Let ti , i = 1,..., n, denote the times at which the periodic premium
payments are made. 3 Let H (t ) denote the set of possible defaults ℓα, α =
1,..., K , that can occur by time t , and let pα(t ) denote the corresponding
probabilities (which are model-dependent). Here ℓα are the dollar amounts of
the defaults. 4 The expected loss L(t ) can be computed as
L(t ) =
K∑
α=1
pα(t ) max(min(ℓα, Ld ) − La ,0)( 11.1)
where La = aM CDO , Ld = dM CDO ,a n d MCDO is the CDO notional
in dollars. 5 From the long tranche investor’s perspective, the mark-to-market
(MTM) value of the tranche, call it M, is given by
M = P − C (11.2)
P = S
n∑
i =1
Di /Delta1i [Mtr − L(ti )] (11.3)
C =
n∑
i =1
Di
[
L(ti ) − L(ti −1)
]
(11.4)
Here: P is the premium leg; C is the contingent (default) leg; S is the spread;
/Delta1i = ti −ti −1; Di is the risk-free discount factor for the payment date ti ;a n d
Mtr = Ld − La is the tranche notional. (Also, t is measured in years, t0 is the
initial time, and L(t0) = 0.) Setting the MTM M = 0 ﬁxes the value of the
spread S = S∗.
Gârleanu ( 2001), Dufﬁe and Huang ( 1996), Dufﬁe and Singleton ( 1997a, b), Fabozzi ( 2006a), Finger
(1999), Frey et al. ( 2001), Gibson ( 2004), Goodman ( 2002), Goodman and Lucas ( 2002), Houdain and
Guegan ( 2006), Hull and White ( 2006, 2010), Jarrow et al. ( 1997), Jarrow and T urnbull ( 1995), Jobst
(2005, 2006a, b, c, 2007), Laurent and Gregory ( 2005), Li ( 2000), Lucas et al. ( 2006), Meissner ( 2008),
Packer and Zhu ( 2005), Prince ( 2005), Schmidt and Ward ( 2002), Schönbucher ( 2003), T avakoli (1998),
Vasicek (2015).
3For simplicity, we can also assume that any default payments are also made at those times.
4If the notional amount of the defaulted credit labeled by α is Mα,t h e n ℓα = Mα(1 − Rα),w h e r e Rα
is the recovery rate (which may be nonzero) of said credit.
5Recall that the attachment a and the detachment d a r em e a s u r e di n% .

11 Structured Assets 183
We can further deﬁne the “risky duration” D of the tranche as the ﬁrst
derivative of the MTM w.r.t. the spread:
M(S) = (S − S∗)
n∑
i =1
Di /Delta1i [Mtr − L(ti )] (11.5)
D = ∂M/∂S =
n∑
i =1
Di /Delta1i [Mtr − L(ti )] (11.6)
The risky duration Dix can also be deﬁned in a similar fashion for a CDS
index.
11.2 Strategy: Carry, Equity Tranche—Index
Hedging
This strategy amounts to buying the equity (lowest quality) tranche and Delta-
hedging it by selling the index. The Delta (i.e., the hedge ratio) is given by 6
/Delta1ix = D
Dix
(11.7)
The premiums received from the equity tranche are higher than the premiums
paid on the short index position. The risk is the exposure to equity tranche
credit events.
11.3 Strategy: Carry, Senior/Mezzanine—Index
Hedging
This strategy amounts to selling a high quality tranche (e.g., senior/mezzanine)
and Delta-hedging the position by buying the index. 7 The Delta is given by
Eq. ( 11.7).
6For some literature on CDO tranche hedging and related topics, see, e.g., Arnsdorf and Halperin ( 2007),
Bielecki et al. ( 2007), Bielecki et al. ( 2008), Carmona and Crépey ( 2010), Cont and Minca ( 2013), Frey
and Backhaus ( 2008, 2010), Giesecke and Weber ( 2006), Herbertsson ( 2008), Houdain and Guegan
(2006), Laurent et al. ( 2011), Walker ( 2008).
7The premiums received from the index are higher than the premiums paid on the short tranche position.
So, this trade is “opposite” to the long equity tranche trade hedged with the index.

184 Z. Kakushadze and J. A. Serur
11.4 Strategy: Carry—Tranche Hedging
This strategy amounts to buying a low quality tranche and Delta-hedging the
position by selling a high quality tranche. The hedge ratio is given by:
/Delta1high = Dlow
Dhigh
(11.8)
Here Dlow and Dhigh are the risky durations of the low and high quality
tranches.
11.5 Strategy: Carry—CDS Hedging
This strategy amounts to buying a low quality tranche and Delta-hedging the
position by selling a single-name CDS with lower premium payments than
the long tranche (instead of the index or a higher quality tranche). The hedge
ratio is given by Eq. ( 11.7) with Dix replaced by the risky duration DCDS of
the CDS:
/Delta1CDS = D
DCDS
(11.9)
11.6 Strategy: CDOs—Curve Trades
As in the case of bonds (see Sect. 5.13), a ﬂattener (steepener) curve trade
involves a simultaneous sale (purchase) of a short-term tranche and a purchase
(sale) of a long-term tranche. Put differently, with a ﬂattener (steepener), the
trader is buying (selling) short-term protection and selling (buying) long-term
protection, i.e., the trader expects the spread curve to ﬂatten (steepen), whereby
the spread between the long-term and short-term tranches decreases (increases).
The carry of the curve trade over the period from time t to time t + /Delta1t can
be deﬁned as follows
C(t, t + /Delta1t ) =
(
M
long Slong − Mshort Sshort
)
/Delta1t (11.10)
where Mlong and Mshort are the long and short tranche notionals, and Slong
and Sshort are the corresponding spreads. The trade can be structured to

11 Structured Assets 185
be dollar-neutral (i.e., notional-neutral, Mlong = Mshort ),8 risky duration-
neutral ( Dlong = Dshort , see Eq. ( 11.6)), carry-neutral ( Mlong Slong =
Mshort Sshort ), etc.9 The P&L of the strategy is given by ( Mlong and Mshort
are the long and short tranche MTMs, see Eq. ( 11.5)):
P&L = Mlong − Mshort (11.11)
11.7 Strategy: Mortgage-Backed Security (MBS)
Trading
This strategy amounts to buying MBS passthroughs 10 and duration-hedging
their interest rate exposure with interest rate swaps. Thus, the main risk of
a passthrough MBS is the prepayment risk, whereby homeowners have an
option to prepay their mortgages. Homeowners reﬁnance their mortgages as
the interest rates drop, which results in negative convexity in the MBS price
as a function of the interest rates (e.g., the 5-year swap rate). The hedge ratios
are model-dependent and a variety of prepayment models can be constructed.
Alternatively one can follow a nonparametric approach whereby using histori-
cal data one estimates the ﬁrst derivative of the passthrough MBS price P w.r.t.
the 5-year swap rate R with the constraint that P is a nonincreasing function
of R (see, e.g., Duarte et al. 2006),
11 employing, e.g., a constrained regression
(see, e.g., Aït-Sahalia and Duarte 2003).
8In this case, for an upward-sloping curve, a ﬂattener (steepener) has positive (negative) carry as Slong >
Sshort (Slong < Sshort ).
9For some literature on curve trades and related topics, see, e.g., Bobey ( 2010), Burtshell et al. ( 2009),
Choro´s-T omczyk et al. ( 2016), Crabbe and Fabozzi ( 2002), Detlefsen and Härdle ( 2013), Hagenstein
et al. ( 2004), Hamerle et al. ( 2012), Hull and White ( 2004), Kakodkar et al. ( 2006), Koopman et al.
(2012), Lin and Shyy ( 2008), Rajan et al. ( 2007).
10An MBS is an asset backed by a pool of mortgages. In a pass-through MBS, which is the most common
MBS type, cash ﬂows are passed from debtors to investors through an intermediary.
11For some additional pertinent literature, see, e.g., Ambrose et al. ( 2004), Biby et al. ( 2001), Bielecki
et al. ( 2011), Boudoukh et al. ( 1997), Brazil ( 1988), Brennan and Schwartz ( 1985), Carron and Hogan
(1988), Chinloy ( 1989), Davidson et al. ( 1988), Dechario et al. ( 2010), Downing et al. ( 2009), Dunn
and McConnell ( 1981a, b), Dynkin et al. ( 2001), Fabozzi ( 2006b), Gabaix et al. ( 2007), Glaeser and
Kallal ( 1997), Hu ( 2001), Longstaff ( 2005), Kau et al. ( 1995), McConnell and Buser ( 2011), McKenzie
(2002), Nothaft et al. ( 1995), Passmore et al. ( 2005), Richard and Roll ( 1989), Schultz ( 2016), Schwartz
and T orous (1989, 1992), Stanton ( 1995), Thibodeau and Giliberto ( 1989), Vickery and Wright ( 2010).

186 Z. Kakushadze and J. A. Serur
References
Aït-Sahalia, Y., & Duarte, J. (2003). Nonparametric Option Pricing Under Shape
Restrictions. Journal of Econometrics , 116 (1–2), 9–47.
Altman, E. I., Brady, B., Resti, A., & Sironi, A. (2005). The Link Between Default and
Recovery Rates: Theory, Empirical Evidence and Implications. Journal of Business ,
78(6), 2203–2228.
Amato, J. D., & Gyntelberg, J. (2005, December). CDS Index T ranches and the
Pricing of Credit Risk Correlations. BIS Quarterly Review , pp. 73–87. Available
online: https://www.bis.org/publ/qtrpdf/r_qt0503g.pdf.
Amato, J. D., & Remolona, E. M. (2003, December). The Credit Spread Puzzle. BIS
Quarterly Review , pp. 51–63. Available online: https://www.bis.org/publ/qtrpdf/
r_qt0312e.pdf .
Ambrose, B., LaCour-Little, M., & Sanders, A. (2004). The Effect of Conforming
Loan Status on Mortgage Yield Spreads: A Loan Level Analysis. Real Estate Eco-
nomics, 32(4), 541–569.
Andersen, L., & Sidenius, J. (2005). Extensions to the Gaussian Copula: Random
Recovery and Random Factor Loadings. Journal of Credit Risk , 1(1), 29–70.
Andersen, L., Sidenius, J., & Basu, S. (2003, November). All Your Hedges in One
Basket. Risk, pp. 67–72.
Arnsdorf, M., & Halperin, I. (2007). BSLP: Markovian Bivariate Spread-Loss Model
for Portfolio Credit Derivatives (Working Paper). Available online: https://arxiv.org/
pdf/0901.3398.
Belkin, B., Suchover, S., & Forest, L. (1998). A One-Parameter Representation of
Credit Risk and T ransition Matrices. Credit Metrics Monitor , 1(3), 46–56.
Biby, J. D., Modukuri, S., & Hargrave, B. (2001). Collateralized Borrowing via Dollar
Rolls. In F . J. Fabozzi (Ed.), The Handbook of Mortgage-Backed Securities (5th ed.).
New York, NY: McGraw-Hill.
Bielecki, T . R., Brigo, D., & Patras, F . (2011). Credit Risk Frontiers: Subprime Crisis,
Pricing and Hedging, CVA, MBS, Ratings, and Liquidity . Hoboken, NJ: Wiley.
Bielecki, T ., Jeanblanc, M., & Rutkowski, M. (2007). Hedging of Basket Credit
Derivatives in the Credit Default Swap Market. Journal of Credit Risk , 3(1), 91–
132.
Bielecki, T ., Vidozzi, A., & Vidozzi, L. (2008). A Markov Copulae Approach to Pricing
and Hedging of Credit Index Derivatives and Ratings T riggered Step-Up Bonds.
Journal of Credit Risk , 4 (1), 47–76.
Bobey, B. (2010). The Effects of Default Correlation on Corporate Bond Credit Spreads
(Working Paper). Available online: https://ssrn.com/abstract=1510170.
Bol, G., Rachev, S. T ., & Würth, R. (Eds.). (2009). Risk Assessment: Decisions in
Banking and Finance . Heidelberg, Germany: Physica-Verlag.
Boscher, H., & Ward, I. (2002, June). Long or Short in CDOs. Risk, pp. 125–129.

11 Structured Assets 187
Boudoukh, J., Whitelaw, R., Richardson, M., & Stanton, R. (1997). Pricing Mortgage-
Backed Securities in a Multifactor Interest Rate Environment: A Multivariate Den-
sity Estimation Approach. Review of Financial Studies , 10(2), 405–446.
Brazil, A. J. (1988). Citicorp’s Mortgage Valuation Model: Option-Adjusted Spreads
and Option-Based Durations. Journal of Real Estate Finance and Economics , 1(2),
151–162.
Brennan, M. J., & Schwartz, E. S. (1985). Determinants of GNMA Mortgage Prices.
Real Estate Economics , 13(3), 209–228.
Burtshell, X., Gregory, J., & Laurent, J.-P . (2009). A Comparative Analysis of CDO
Pricing Models Under the Factor Copula Framework. Journal of Derivatives, 16 (4),
9–37.
Carmona, R., & Crépey, S. (2010). Particle Methods for the Estimation of Credit Port-
folio Loss Distributions. International Journal of Theoretical and Applied Finance ,
13(4), 577–602.
Carron, A. S., & Hogan, M. (1988). The Option Valuation Approach to Mortgage
Pricing. Journal of Real Estate Finance and Economics , 1(2), 131–149.
Chinloy, P . (1989). The Probability of Prepayment. Journal of Real Estate Finance and
Economics, 2(4), 267–283.
Choro´s-T omczyk, B., Härdle, W . K., & Okhrin, O. (2016). A Semiparametric Factor
Model for CDO Surfaces Dynamics. Journal of Multivariate Analysis , 146, 151–
163.
Cont, R., & Minca, A. (2013). Recovering Portfolio Default Intensities Implied by
CDO Quotes. Mathematical Finance , 23(1), 94–121.
Cousin, A., & Laurent, J. (2012). Dynamic Hedging of Synthetic CDO T ranches:
Bridging the Gap Between Theory and Practice. In T . R. Bielecki, D. Brigo, & F .
Patras (Eds.), Credit Risk Frontiers (Chapter 6). Hoboken, NJ: Wiley.
Crabbe, L. E., & Fabozzi, F . J. (2002).Corporate Bond Portfolio Management. Hoboken,
NJ: Wiley.
Das, S. (2005). Credit Derivatives: T rading & Management of Credit & Default Risk
(3rd ed.). Hoboken, NJ: Wiley.
Davidson, A. S., Herskovitz, M. D., & Van Drunen, L. D. (1988). The Reﬁnancing
Threshold Pricing Model: An Economic Approach to Valuing MBS. Journal of Real
Estate Finance and Economics , 1(2), 117–130.
Davis, M., & Lo, V . (2001). Infectious Defaults. Quantitative Finance, 1(4), 382–387.
Dechario, T ., Mosser, P ., T racy, J., Vickery, J., & Wright, J. (2010). AP r i v a t e
Lender Cooperative Model for Residential Mortgage Finance (Federal Reserve Bank
of New York Staff Reports, No. 466). Available online: https://www.newyorkfed.
org/medialibrary/media/research/staff_reports/sr466.pdf .
Detlefsen, K., & Härdle, W . K. (2013). Variance Swap Dynamics. Quantitative
Finance, 13(5), 675–685.
Ding, J. J., & Sherris, M. (2011). Comparison of Market Models for Measuring and
Hedging Synthetic CDO T ranche Spread Risks. European Actuarial Journal, 1(S2),
261–281.

188 Z. Kakushadze and J. A. Serur
Douglas, R. (Ed.). (2007). Credit Derivative Strategies: New Thinking on Managing
Risk and Return . New York, NY: Bloomberg Press.
Downing, C., Jaffee, D., & Wallace, N. (2009). Is the Market for Mortgage-Backed
Securities a Market for Lemons? Review of Financial Studies , 22(7), 2457–2494.
Duarte, J., Longstaff, F . A., & Yu, F . (2006). Risk and Return in Fixed-Income Arbi-
trage: Nickels in Front of a Steamroller? Review of Financial Studies , 20(3), 769–
811.
Dufﬁe, D. (2004, April). Time to Adapt Copula Methods for Modelling Credit Risk
Correlation. Risk, p. 77.
Dufﬁe, D., & Gârleanu, N. (2001). Risk and Valuation of Collateralized Debt Obli-
gations. Financial Analysts Journal , 57 (1), 41–59.
Dufﬁe, D., & Huang, M. (1996). Swap Rates and Credit Quality. Journal of Finance ,
51(2), 921–949.
Dufﬁe, D., & Singleton, K. J. (1997a). Modeling T erm Structures of Defaultable
Bonds. Review of Financial Studies , 12(4), 687–720.
Dufﬁe, D., & Singleton, K. J. (1997b). An Econometric Model of the T erm Structure
of Interest Rate Swap Yields. Journal of Finance , 52(4), 1287–1321.
Dunn, K. B., & McConnell, J. J. (1981a). A Comparison of Alternative Models for
Pricing GNMA Mortgage-Backed Securities. Journal of Finance , 36 (2), 471–484.
Dunn, K. B., & McConnell, J. J. (1981b). Valuation of GNMA Mortgage-Backed
Securities. Journal of Finance , 36 (3), 599–616.
Dynkin, L., Hyman, J., Konstantinovsky, V ., & Roth, N. (2001). Building an MBS
Index: Conventions and Calculations. In F . J. Fabozzi (Ed.), The Handbook of
Mortgage-Backed Securities (5th ed.). New York, NY: McGraw-Hill.
Fabozzi, F . J. (2006a). Fixed Income Mathematics: Analytical & Statistical T echniques .
New York, NY: McGraw-Hill.
Fabozzi, F . J. (Ed.). (2006b). The Handbook of Mortgage-Backed Securities .N e wY o r k ,
NY: McGraw-Hill.
Finger, C. C. (1999). Conditional Approaches for Credit Metrics Portfolio Distribu-
tions. Credit Metrics Monitor , 2(1), 14–33.
Frey, R., McNeil, A., & Nyfeler, N. (2001, October). Copulas and Credit Models.
Risk, pp. 111–114.
Frey, R., & Backhaus, J. (2008). Pricing and Hedging of Portfolio Credit Derivatives
with Interacting Default Intensities. International Journal of Theoretical and Applied
Finance, 11(6), 611–634.
Frey, R., & Backhaus, J. (2010). Dynamic Hedging of Synthetic CDO T ranches with
Spread Risk and Default Contagion. Journal of Economic Dynamics and Control ,
34 (4), 710–724.
Gabaix, X., Krishnamurthy, A., & Vigneron, O. (2007). Limits of Arbitrage: Theory
and Evidence from the Mortgage-Backed Securities Market. Journal of Finance ,
62(2), 557–595.
Gibson, M. S. (2004). Understanding the Risk of Synthetic CDOs (Finance and
Economics Discussion Series (FEDS), Paper No. 2004-36). Washington, DC:

11 Structured Assets 189
Board of Governors of the Federal Reserve System. Available online: https://www.
federalreserve.gov/pubs/feds/2004/200436/200436pap.pdf.
Giesecke, K., & Weber, S. (2006). Credit Contagion and Aggregate Losses. Journal of
Economic Dynamics and Control , 30(5), 741–767.
Glaeser, E. L., & Kallal, H. D. (1997). Thin Markets, Asymmetric Information, and
Mortgage-Backed Securities. Journal of Financial Intermediation , 6 (1), 64–86.
Goodman, L. S. (2002). Synthetic CDOs: An Introduction. Journal of Derivatives ,
9(3), 60–72.
Goodman, L. S., & Lucas, D. J. (2002). And When CDOs PIK? Journal of Fixed
Income, 12(1), 96–102.
Hagenstein, F ., Mertz, A., & Seifert, J. (2004). Investing in Corporate Bonds and Credit
Risk. London, UK: Palgrave Macmillan.
Hamerle, A., Igl, A., & Plank, K. (2012). Correlation Smile, Volatility Skew, and
Systematic Risk Sensitivity of T ranches. Journal of Derivatives , 19(3), 8–27.
Herbertsson, A. (2008). Pricing Synthetic CDO T ranches in a Model with Default
Contagion Using the Matrix-Analytic Approach. Journal of Credit Risk , 4 (4), 3–35.
Houdain, J. P ., & Guegan, D. (2006). Hedging T ranches Index Products: Illustration
of Model Dependency. ICFAI Journal of Derivatives Markets , 4, 39–61.
Hu, J. (2001). Basics of Mortgage-Backed Securities (2nd ed.). Hoboken, NJ: Wiley.
Hull, J. C., & White, A. D. (2004). Valuation of a CDO and an nth to Default CDS
Without Monte Carlo Simulation. Journal of Derivatives , 12(2), 8–23.
Hull, J. C., & White, A. D. (2006). Valuing Credit Derivatives Using an Implied
Copula Approach. Journal of Derivatives , 14 (2), 8–28.
Hull, J. C., & White, A. D. (2010). An Improved Implied Copula Model and Its
Application to the Valuation of Bespoke CDO T ranches. Journal of Investment
Management, 8(3), 11–31.
Jarrow, R., Lando, D., & T urnbull, S. (1997). A Markov Model for the T erm Structure
of Credit Spreads. Review of Financial Studies , 10(2), 481–523.
Jarrow, R. A., & T urnbull, S. M. (1995). Pricing Derivatives on Financial Securities
Subject to Credit Risk. Journal of Finance , 50(1), 53–85.
Jobst, A. (2005). T ranche Pricing in Subordinated Loan Securitization. Journal of
Structured Finance, 11(2), 64–96.
Jobst, A. (2006a). European Securitization: A GARCH Model of Secondary Market
Spreads. Journal of Structured Finance , 12(1), 55–80.
Jobst, A. (2006b). Sovereign Securitization in Emerging Markets. Journal of Structured
Finance, 12(3), 2–13.
Jobst, A. (2006c). Correlation, Price Discovery and Co-movement of ABS and Equity.
Derivatives Use, T rading & Regulation , 12(1–2), 60–101.
Jobst, A. (2007). A Primer on Structured Finance. Journal of Derivatives & Hedge
Funds, 13(3), 199–213.
Kakodkar, A., Galiani, S., Jónsson, J. G., & Gallo, A. (2006). Credit Derivatives
Handbook 2006—Vol. 2: A Guide to the Exotics Credit Derivatives Market .N e w
York, NY: Credit Derivatives Strategy, Merrill Lynch.

190 Z. Kakushadze and J. A. Serur
Kau, J. B., Keenan, D. C., Muller, W . J., & Epperson, J. F . (1995). The Valuation
at Origination of Fixed-Rate Mortgages with Default and Prepayment. Journal of
Real Estate Finance and Economics , 11(1), 5–36.
Koopman, S. J., Lucas, A., & Schwaab, B. (2012). Dynamic Factor Models with
Macro, Frailty, and Industry Effects for U.S. Default Counts: The Credit Crisis of
2008. Econometric Reviews , 30(4), 521–532.
Laurent, J.-P ., Cousin, A., & Fermanian, J. D. (2011). Hedging Default Risks of CDOs
in Markovian Contagion Models. Quantitative Finance , 11(12), 1773–1791.
Laurent, J.-P ., & Gregory, J. (2005). Basket Default Swaps, CDOs and Factor Copulas.
Journal of Risk , 7 (4), 8–23.
Li, D. X. (2000). On Default Correlation: A Copula Function Approach. Journal of
Fixed Income, 9(4), 43–54.
Lin, S.-Y., & Shyy, G. (2008). Credit Spreads, Default Correlations and CDO T ranching:
New Evidence from CDS Quotes (Working Paper). Available online: https://ssrn.
com/abstract=496225.
Longstaff, F . (2005). Borrower Credit and the Valuation of Mortgage-Backed Securi-
ties. Real Estate Economics , 33(4), 619–661.
Lucas, D. J., Goodman, L. S., & Fabozzi, F . J. (Eds.). (2006). Collateralized Debt
Obligations: Structures and Analysis . Hoboken, NJ: Wiley.
McConnell, J. J., & Buser, S. A. (2011). The Origins and Evolution of the Market for
Mortgage-Backed Securities. Annual Review of Financial Economics , 3, 173–192.
McKenzie, J. A. (2002). A Reconsideration of the Jumbo/Non-Jumbo Mortgage Rate
Differential. Journal of Real Estate Finance and Economics , 25(2–3), 197–213.
Meissner, G. (Ed.). (2008). The Deﬁnitive Guide to CDOs .L o n d o n ,U K :I n c i s i v e
Media.
Nothaft, F . E., Lekkas, V ., & Wang, G. H. K. (1995). The Failure of the Mortgage-
Backed Futures Contract. Journal of Futures Markets , 15(5), 585–603.
Packer, F ., & Zhu, H. (2005, March). Contractual T erms and CDS Pricing. BIS
Quarterly Review , pp. 89–100. Available online: https://www.bis.org/publ/qtrpdf/
r_qt0503h.pdf .
Passmore, W ., Sherlund, S. M., & Burgess, G. (2005). The Effect of Housing
Government-Sponsored Enterprises on Mortgage Rates. Real Estate Economics ,
33(3), 427–463.
Prince, J. T . (2005). Investing in Collateralized Debt Obligations. In CFA Institute
Conference Proceedings (Vol. 2005, No. 1, pp. 52–61).
Rajan, A., McDermott, G., & Roy, R. (Eds.). (2007). The Structured Credit Handbook.
Hoboken, NJ: Wiley.
Richard, S. F ., & Roll, R. (1989). Prepayments on Fixed-Rate Mortgage-Backed Secu-
rities. Journal of Portfolio Management , 15(3), 73–82.
Schmidt, W ., & Ward, I. (2002, January). Pricing Default Baskets. Risk, pp. 111–114.
Schönbucher, P . J. (2003). Credit Derivatives Pricing Models
. Hoboken, NJ: Wiley.
Schultz, G. M. (2016). Investing in Mortgage-Backed and Asset-Backed Securities: Finan-
cial Modeling with R and Open Source Analytics + Website . Hoboken, NJ: Wiley.

11 Structured Assets 191
Schwartz, E. S., & T orous, W . N. (1989). Prepayment and the Valuation of Mortgage-
Backed Securities. Journal of Finance , 44 (2), 375–392.
Schwartz, E. S., & T orous, W . N. (1992). Prepayment, Default, and the Valuation of
Mortgage Pass-Through Securities. Journal of Business , 65(2), 221–239.
Stanton, R. (1995). Rational Prepayment and the Valuation of Mortgage-Backed
Securities. Review of Financial Studies , 8(3), 677–708.
T avakoli, J. M. (1998).Credit Derivatives & Synthetic Structures: A Guide to Instruments
and Applications (2nd ed.). Hoboken, NJ: Wiley.
Thibodeau, T . G., & Giliberto, S. M. (1989). Modeling Conventional Residential
Mortgage Reﬁnancing. Journal of Real Estate Finance and Economics , 2(4), 285–
299.
Vasicek, O. A. (2015). Probability of Loss on Loan Portfolio. In O. A. Vasicek (Ed.),
Finance, Economics and Mathematics (Chapter 17). Hoboken, NJ: Wiley.
Vickery, J., & Wright, J. (2010). TBA T rading and Liquidity in the Agency
MBS Market (Federal Reserve Bank of New York Staff Reports, No. 468).
Available online: https://www.newyorkfed.org/medialibrary/media/research/staff_
reports/sr468.pdf .
Walker, M. B. (2008). The Static Hedging of CDO T ranche Correlation Risk. Inter-
national Journal of Computer Mathematics , 86 (6), 940–954.
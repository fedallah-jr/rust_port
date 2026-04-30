# Chapter 12: Convertibles

12
Convertibles
12.1 Strategy: Convertible Arbitrage
A convertible bond is a hybrid security with an embedded option to convert the
bond (a ﬁxed-income instrument) to a preset number (knows as the conversion
ratio) of the issuer’s stock (an equity instrument) when, e.g., the stock price
reaches a preset level (known as the conversion price). Empirically, convertibles
at the issuance tend to be undervalued relative to their “fair” value.
1 This gives
rise to arbitrage opportunities. A convertible arbitrage strategy amounts to buy-
ing a convertible bond and simultaneously shorting h units of the underlying
stock, where the hedge ratio is given by
h = /Delta1× C (12.1)
/Delta1= ∂V /∂S (12.2)
Here: C is the conversion ratio; V is the value of the conversion option (which
is model-dependent); S is the underlying stock price; and /Delta1is the (model-
dependent) Delta of the conversion option. 2 T ypically, the position is held for
1For some literature on convertible bonds and related topics, see, e.g., Agarwal et al. ( 2011), Ammann
et al. ( 2003, 2010), Batta et al. ( 2010), Brennan and Schwartz ( 1988), Brown et al. ( 2012), Calamos
(2003), Chan and Chen ( 2007), Choi et al. ( 2009, 2010), De Jong et al. ( 2011), Duca et al. ( 2012),
Dutordoir et al. ( 2014), Grundy and V erwijmeren ( 2016), Henderson ( 2005), Henderson and T ookes
(2012), Ingersoll ( 1977), Kang and Lee ( 1996), King ( 1986), King and Mauer ( 2014), Korkeamaki and
Michael (2013) ,L e w i se ta l .(1999), Lewis and V erwijmeren ( 2011), Loncarski et al. ( 2006, 2009), Mayers
(1998), Ryabkov ( 2015), Stein ( 1992), T siveriotis and Fernandes ( 1998), van Marle and V erwijmeren
(2017), Zabolotnyuk et al. ( 2010).
2The Delta itself changes with the stock price S. T o account for this, the option Gamma can be used as
in section “ Strategy: V olatility Risk Premium with Gamma Hedging ” in Chapter 7 (Gamma hedging).
© The Author(s) 2018
Z. Kakushadze and J. A. Serur, 151 T rading Strategies,
https://doi.org/10.1007/978-3-030-02792-6_12
193

194 Z. Kakushadze and J. A. Serur
6–12 months starting at the issuance date of the convertible and the hedge
ratio is updated daily.
12.2 Strategy: Convertible Option-Adjusted
Spread
This strategy amounts to simultaneously buying and selling two different con-
vertible bonds of the same issuer. The long position is in a bond with a higher
option-adjusted spread (OAS), and the short position is in a bond with a lower
OAS (see, e.g., Calamos 2003). Then the trade is proﬁtable if these two spreads
converge.
The OAS can be calculated as follows (see, e.g., Hull 2012).
3 A straightfor-
ward (but not the only) 4 way to compute the price PC of the convertible bond
is to assume that
PC = PB + V (12.3)
where PB is the price of the straight bond (without the embedded option),
and V is the value of the conversion option, which is a call option. PB is
computed via the standard discounting of the future cash ﬂows of the bond.
On the other hand, V depends on the risk-free interest rate curve. At the initial
iteration, V is computed (using a pricing model for the call option) assuming
the zero-coupon government T reasury curve as the risk-free interest rate curve.
This initial iteration V (0) may not coincide with Pmkt
C − PB ,w h e r e Pmkt
C is
the market price of the convertible bond. Then one iteratively (e.g., using the
bisection method) parallel shifts the input T reasury curve until V computed
using the so-shifted curve is such that V = Pmkt
C − PB . The curve parallel
shift obtained via this iterative procedure is the OAS.
3For some additional literature related to OAS (mostly focused on applications to MBS), see, e.g.,
Boyarchenko et al. ( 2014), Brazil ( 1988), Brown ( 1999), Cerrato and Djennad ( 2008), Dong et al. ( 2009),
Hayre ( 1990), Huang and Kong ( 2003), Levin and Davidson ( 2005), Liu and Xu ( 1998), Stroebel and
T aylor (2012), Windas ( 2007).
4For some literature on convertible bond pricing, see, e.g., Ayache et al. ( 2003), Batten et al. ( 2014), Bren-
nan and Schwartz ( 1977), Finnerty and T u ( 2017), Ingersoll ( 1977), Kang and Lee ( 1996), King ( 1986),
Kwok ( 2014), McConnell and Schwartz ( 1986), Milanov et al. ( 2013), Park et al. ( 2018), Sörensson
(1993), T siveriotis and Fernandes ( 1998), Xiao ( 2013), Zabolotnyuk et al. ( 2010).

12 Convertibles 195
References
Agarwal, V ., Fung, W . H., Loon, Y. C., & Naik, N. Y. (2011). Risk and Return in
Convertible Arbitrage: Evidence from the Convertible Bond Market. Journal of
Empirical Finance , 18(2), 175–194.
Ammann, M., Kind, A., & Seiz, R. (2010). What Drives the Performance of
Convertible-Bond Funds? Journal of Banking & Finance , 34 (11), 2600–2613.
Ammann, M., Kind, A., & Wilde, C. (2003). Are Convertible Bonds Underpriced?
An Analysis of the French Market. Journal of Banking & Finance , 27 (4), 635–653.
Ayache, E., Forsyth, P . A., & V etzal, K. R. (2003). Valuation of Convertible Bonds
with Credit Risk. Journal of Derivatives , 11(1), 9–29.
Batta, G., Chacko, G., & Dharan, B. (2010). A Liquidity-Based Explanation of Con-
vertible Arbitrage Alphas. Journal of Fixed Income , 20 (1), 28–43.
Batten, J. A., Khaw, K., & Young, M. R. (2014). Convertible Bond Pricing Models.
Journal of Economic Surveys , 28(5), 775–803.
Boyarchenko, N., Fuster, A., & Lucca, D. O. (2014). Understanding Mort-
gage Spreads (Federal Reserve Bank of New York Staff Reports, No. 674).
Available online: https://www.newyorkfed.org/medialibrary/media/research/staff_
reports/sr674.pdf .
Brazil, A. J. (1988). Citicorp’s Mortgage Valuation Model: Option-Adjusted Spreads
and Option-Based Durations. Journal of Real Estate Finance and Economics , 1(2),
151–162.
Brennan, M. J., & Schwartz, E. S. (1977). Convertible Bonds: Valuation and Optimal
Strategies for Call and Conversion. Journal of Finance , 32(5), 1699–1715.
Brennan, M. J., & Schwartz, E. S. (1988). The Case for Convertibles. Journal of
Applied Corporate Finance , 1(2), 55–64.
Brown, D. (1999). The Determinants of Expected Returns on Mortgage-Backed Secu-
rities: An Empirical Analysis of Option-Adjusted Spreads. Journal of Fixed Income ,
9 (2), 8–18.
Brown, S. J., Grundy, B. D., Lewis, C. M., & V erwijmeren, P . (2012). Convertibles
and Hedge Funds as Distributors of Equity Exposure. Review of Financial Studies ,
25 (10), 3077–3112.
Calamos, N. P . (2003). Convertible Arbitrage: Insights and T echniques for Successful
Hedging. Hoboken, NJ: Wiley.
Cerrato, M., & Djennad, A. (2008). Dynamic Option Adjusted Spread and the Value
of Mortgage Backed Securities (Working Paper). Available online: https://www.gla.
ac.uk/media/media_71226_en.pdf .
Chan, A. W . H., & Chen, N.-F . (2007). Convertible Bond Underpricing: Rene-
gotiable Covenants, Seasoning, and Convergence. Management Science , 53(11),
1793–1814.
Choi, D., Getmansky, M., Henderson, B., & T ookes, H. (2010). Convertible Bond
Arbitrageurs as Suppliers of Capital. Review of Financial Studies , 23(6), 2492–2522.

196 Z. Kakushadze and J. A. Serur
Choi, D., Getmansky, M., & T ookes, H. (2009). Convertible Bond Arbitrage, Liquid-
ity Externalities, and Stock Prices. Journal of Financial Economics , 91(2), 227–251.
De Jong, A., Dutordoir, M., & V erwijmeren, P . (2011). Why Do Convertible Issuers
Simultaneously Repurchase Stock? An Arbitrage-Based Explanation. Journal of
Financial Economics , 100 (1), 113–129.
Dong, J.-C., Liu, J.-X., Wang, C.-H., Yuan, H., & Wang, W .-J. (2009). Pricing
Mortgage-Backed Security: An Empirical Analysis. Systems Engineering—Theory &
Practice, 29 (12), 46–52.
Duca, E., Dutordoir, M., V eld, C., & V erwijmeren, P . (2012). Why Are Convertible
Bond Announcements Associated with Increasingly Negative Issuer Stock Returns?
An Arbitrage Based Explanation. Journal of Banking & Finance , 36 (11), 2884–
2899.
Dutordoir, M., Lewis, C. M., Seward, J., & V eld, C. (2014). What We Do and Do
Not Know About Convertible Bond Financing. Journal of Corporate Finance , 24,
3–20.
Finnerty, J. D., & T u, M. (2017). Valuing Convertible Bonds: A New Approach.
Business Valuation Review , 36 (3), 85–102.
Grundy, B. D., & V erwijmeren, P . (2016). Disappearing Call Delay and Dividend-
Protected Convertible Bonds. Journal of Finance , 71(1), 195–224.
Hayre, L. S. (1990). Understanding Option-Adjusted Spreads and Their Use. Journal
of Portfolio Management , 16 (4), 68–69.
Henderson, B. J. (2005). Convertible Bonds: New Issue Performance and Arbitrage
Opportunities. Ph.D. thesis, University of Illinois, Urbana-Champaign, IL.
Henderson, B. J., & T ookes, H. (2012). Do Investment Banks’ Relationships with
Investors Impact Pricing? The Case of Convertible Bond Issues. Management Sci-
ence, 58(2), 2272–2291.
Huang, J.-Z., & Kong, W . (2003). Explaining Credit Spread Changes: New Evidence
From Option-Adjusted Bond Indexes. Journal of Derivatives , 11(1), 30–44.
Hull, J. C. (2012). Options, Futures and Other Derivatives . Upper Saddle River, NJ:
Prentice Hall.
Ingersoll, J. (1977). A Contingent-Claims Valuation of Convertible Securities. Journal
of Financial Economics , 4 (3), 289–322.
Kang, J. K., & Lee, Y. W . (1996). The Pricing of Convertible Debt Offerings. Journal
of Financial Economics , 41(2), 231–248.
King, R. (1986). Convertible Bond Valuation: An Empirical T est. Journal of Financial
Research, 9 (1), 53–69.
King, T . H. D., & Mauer, D. C. (2014). Determinants of Corporate Call Policy for
Convertible Bonds. Journal of Corporate Finance , 24, 112–134.
Korkeamaki, T ., & Michael, T . B. (2013). Where Are They Now? An Analysis of the
Life Cycle of Convertible Bonds. Financial Review , 48(3), 489–509.
Kwok, Y. K. (2014). Game Option Models of Convertible Bonds: Determinants of
Call Policies. Journal of Financial Engineering , 1(4), 1450029.
Levin, A., & Davidson, A. (2005). Prepayment Risk- and Option-Adjusted Valuation
of MBS. Journal of Portfolio Management , 31(4), 73–85.

12 Convertibles 197
Lewis, C. M., Rogalski, R. J., & Seward, J. K. (1999). Is Convertible Debt a Substitute
for Straight Debt or for Common Equity? Financial Management , 28(3), 5–27.
Lewis, C. M., & V erwijmeren, P . (2011). Convertible Security Design and Contract
Innovation. Journal of Corporate Finance , 17 (4), 809–831.
Liu, J.-G., & Xu, E. (1998). Pricing of Mortgage-Backed Securities with Option-
Adjusted Spread. Managerial Finance , 24 (9–10), 94–109.
Loncarski, I., ter Horst, J. R., & V eld, C. H. (2006). The Convertible Arbitrage Strategy
Analyzed (Working Paper). Available online: https://pure.uvt.nl/ws/ﬁles/779871/
98.pdf .
Loncarski, I., ter Horst, J. R., & V eld, C. H. (2009). The Rise and Demise of the
Convertible Arbitrage Strategy. Financial Analysts Journal , 65 (5), 35–50.
Mayers, D. (1998). Why Firms Issue Convertible Bonds: The Matching of Financial
and Real Investment Options. Journal of Financial Economics , 47 (1), 83–102.
McConnell, J. J., & Schwartz, E. S. (1986). LYON T aming. Journal of Finance , 41(3),
561–577.
Milanov, K., Kounchev, O., Fabozzi, F . J., Kim, Y. S., & Rachev, S. T . (2013). A
Binomial-T ree Model for Convertible Bond Pricing. Journal of Fixed Income , 22(3),
79–94.
Park, K., Jung, M., & Lee, S. (2018). Credit Ratings and Convertible Bond Prices: A
Simulation-Based Valuation. European Journal of Finance , 24 (12), 1001–1025.
Ryabkov, N. (2015). Hedge Fund Price Pressure in Convertible Bond Markets (Working
Paper). Available online: https://ssrn.com/abstract=2539929.
Sörensson, T . (1993). T wo Methods for Valuing Convertible Bonds—A Comparison.
Scandinavian Journal of Management , 9 (S1), 129–139.
Stein, J. C. (1992). Convertible Bonds as Backdoor Equity Financing. Journal of
Financial Economics , 32(1), 3–21.
Stroebel, J., & T aylor, J. B. (2012). Estimated Impact of the Federal Reserve’s
Mortgage-Backed Securities Purchase Program. International Journal of Central
Banking, 8(2), 1–42.
T siveriotis, K., & Fernandes, C. (1998). Valuing Convertible Bonds with Credit Risk.
Journal of Fixed Income , 8(2), 95–102.
van Marle, M., & V erwijmeren, P . (2017). The Long and the Short of Convertible
Arbitrage: An Empirical Examination of Arbitrageurs’ Holding Periods. Journal of
Empirical Finance , 44, 237–249.
Windas, T . (2007). An Introduction to Option-Adjusted Spread Analysis (T . Miller, Ed.,
Revised and Expanded Third Edition). Princeton, NJ: Bloomberg Press.
Xiao, T . (2013). A Simple and Precise Method for Pricing Convertible Bond with
Credit Risk. Journal of Derivatives & Hedge Funds , 19 (4), 259–277.
Zabolotnyuk, Y., Jones, R., & V eld, C. (2010). An Empirical Comparison of Con-
vertible Bond Valuation Models. Financial Management , 39 (2), 675–706.
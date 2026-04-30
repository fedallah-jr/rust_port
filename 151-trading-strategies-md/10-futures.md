# Chapter 10: Futures

10
Futures
10.1 Strategy: Hedging Risk with Futures
Exposures to certain risks can be mitigated by hedging with futures. E.g., a
grain trader who at time t anticipates that he or she will need to buy (sell)
X tons of soy at a later time T can hedge the risk of soy prices increasing
(decreasing) between t and T by buying (selling) at time t a futures contract
with the delivery date T for the desired amount of soy. This simple strategy
can have tweaks and variations. 1
Strategy: Cross-Hedging
Sometimes a futures contract for the asset to be hedged may not be available.
In such cases, the trader may be able to hedge using a futures contract for
another asset with similar characteristics.
2 At maturity T , the payoff of the
cross-hedged position established at time t (assuming the short futures position
1For some literature on hedging with futures, see, e.g., Ahmadi et al. ( 1986), Cheung et al. ( 1990),
Ederington (1979), Géczy et al. ( 1997), Ghosh ( 1993), Grant ( 2016), Hanly et al. ( 2018), Lebeck ( 1978),
Lien and Tse ( 2000), Mun ( 2016), Wolf ( 1987), Working ( 1953).
2For some literature on cross-hedging with futures, see, e.g., Anderson and Danthine ( 1981), Ankirchner et
al. (2012), Ankirchner and Heyne ( 2012), Benet ( 1990), Blake and Catlett ( 1984), Blank ( 1984), Brooks
et al. ( 2007), Chen and Sutcliffe ( 2007), Dahlgran ( 2000), DeMaskey ( 1997), DeMaskey and Pearce
(1998), Foster and Whiteman ( 2002), Franken and Parcell ( 2003), Hartzog ( 1982), Lafuente ( 2013),
McEnally and Rice ( 1979), Mun and Morgan ( 1997).
© The Author(s) 2018
Z. Kakushadze and J. A. Serur, 151 T rading Strategies,
https://doi.org/10.1007/978-3-030-02792-6_10
165

166 Z. Kakushadze and J. A. Serur
with the unit hedge ratio) is given by:
S(T ) − F (T , T ) + F (t, T ) =
[S∗(T ) − F (T , T )]+[ S(T ) − S∗(T )]+ F (t, T )( 10.1)
Here: the subscript ∗ indicates that the underlying asset of the futures contract
is different from the hedged asset; S(t ) is the spot price; F (t, T ) is the futures
price; the ﬁrst term on the r.h.s. represents the basis risk stemming from the
difference at delivery between the futures and the spot prices; and the second
term represents the difference twixt the two underlying prices. In practice,
the optimal hedge ratio may not be 1 and can be estimated via, e.g., a serial
regression or some other method.
3
Strategy: Interest Rate Risk Hedging
Fixed-income assets are sensitive to interest rate variations (see Chapter 5)a n d
futures contracts can be used to hedge the interest rate risk. A long (short)
hedge position amounts to buying (selling) interest rate futures in order to
hedge against an increase (decrease) in the price of the underlying asset, i.e.,
a decrease (increase) in the interest rates.
4 The corresponding P&L ( PL (t, T )
for the long hedge and PS (t, T ) for the short hedge, assuming the position
is established at t = 0 with the unit hedge ratio and the maturity is T )i s
given by:
PL (t, T ) = B(0, T ) − B(t, T )( 10.2)
PS (t, T ) = B(t, T ) − B(0, T )( 10.3)
B(t, T ) = S(t ) − F (t, T )( 10.4)
where B(t, T ) is the futures basis. In practice, the hedge ratio may not be 1. If
a hedge is against a bond in the futures delivery basket, 5 then the conversion
3For various optimal hedge ratio techniques, see, e.g., Baillie and Myers ( 1991), Brooks and Chong ( 2001),
Brooks et al. ( 2002), Cecchetti et al. ( 1988), Davis ( 2006), Holmes ( 1996), Lien ( 1992, 2004), Lien and
Luo ( 1993), Lindahl ( 1992), Low et al. ( 2002), Kroner and Sultan ( 1993), Monoyios ( 2004), Moosa
(2003), Myers ( 1991).
4For some literature on hedging the interest rate risk with futures, see, e.g., Booth et al. ( 1984), Briys
and Solnik ( 1992), ˇCerovi´c and Pepi´c( 2011), Clare et al. ( 2000), Fortin and Khoury ( 1984), Gay et al.
(1983), Hilliard ( 1984), Hilliard and Jordan ( 1989), Ho and Saunders ( 1983), Kolb and Chiang ( 1982),
Lee and Oh ( 1993), Pepi´c( 2014), Picou ( 1981), T rainer (1983), Yawitz and Marshall ( 1985), Yeutter and
Dew ( 1982).
5T ypically, an interest rate futures contract allows not just one bond but any bond from a predeﬁned array
of bonds (with varying maturities, coupons, etc.) to be delivered. Hence the use of the conversion factor

10 Futures 167
factor model 6 is commonly used to compute the hedge ratio hC :
hC = C MB
MF
(10.5)
where MB is the nominal value of the bond, MF is the nominal value of the
futures, and C is the conversion factor. Unlike the conversion factor model,
the modiﬁed duration hedge ratio h D can be used for both deliverable and
non-deliverable bonds:
h D = β DB
DF
(10.6)
where DB is the dollar duration 7 of the bond, DF is the dollar duration of the
futures, and β (which is often set to 1) is the change in the bond yield relative
to the change in the futures yield, both taken for a given change in the risk-free
rate.8
10.2 Strategy: Calendar Spread
A bull (bear) futures spread amounts to buying (selling) a near-month futures
and selling (buying) a deferred-month futures. This reduces exposure to the
overall market volatility and allows to focus more on the fundamentals. Thus,
for commodity futures, for the most part, near-month contracts react to supply
and demand more than deferred-month contracts. Therefore, if the trader
expects low (high) supply and high (low) demand, then the trader can make a
bet with a bull (bear) spread.
9
(see below) deﬁned as follows (Hull 2012): “The conversion factor for a bond is set equal to the quoted
price the bond would have per dollar of principal on the ﬁrst day of the delivery month on the assumption
that the interest rate for all maturities equals 6% per annum (with semiannual compounding).”
6The conversion factor model applies only to futures contracts that use conversion factors, such as T-bond
and T-note futures.
7Recall that the dollar duration equals the price times the modiﬁed duration.
8The factor β can be estimated based on the historical data. For some literature on interest rate futures
hedge ratios and related topics, see, e.g., Chang and Fang ( 1990), Chen et al. ( 2005), Daigler and Copper
(1998), Falkenstein and Hanweck ( 1996), Fisher and Weil ( 1971), Gay and Kolb ( 1983), Geske and
Pieptea (1987), Grieves and Mann ( 2004), Grieves and Marcus ( 2005), Hegde ( 1982), Kolb and Chiang
(1981), Kuberek and Peﬂey ( 1983), Landes et al. ( 1985), Pitts ( 1985), Rendleman ( 1999), T oevs and
Jacob (1986), Viswanath ( 1993).
9For some literature on futures calendar spreads and related topics, see, e.g., Abken ( 1989), Adrangi et
al. ( 2006), Barrett and Kolb ( 1995), Bernstein ( 1990), Bessembinder ( 1992, 1993), Bessembinder and
Chan ( 1992), Billingsley and Chance ( 1988), Castelino and Vora ( 1984), Cole et al. ( 1999), Daigler

168 Z. Kakushadze and J. A. Serur
10.3 Strategy: Contrarian Trading
(Mean-Reversion)
This strategy is similar to the mean-reversion strategy discussed in Sect. 3.9.
Within a given universe of futures labeled by i = 1,..., N , the “market index”
return is calculated as an equally weighted average:
Rm = 1
N
N∑
i =1
Ri (10.7)
where Ri are the individual futures returns (typically over the last one week).
The capital allocation weights wi then are given by
wi =− γ [ Ri − Rm ] (10.8)
where γ> 0 is ﬁxed via the normalization condition
N∑
i =1
|wi |= 1 (10.9)
Note that the strategy is automatically dollar-neutral. It amounts to buying
losers and selling winners w.r.t. the market index (see, e.g., Wang and Yu
2004).10 As in the case of equities, the simple weighting scheme given by Eq.
(10.8) is prone to overinvesting in volatile futures, which can be mitigated by
suppressing wi by 1/σi or 1/σ2
i ,w h e r e σi are the historical volatilities. The
portfolio is rebalanced weekly.
Strategy: Contrarian Trading—Market Activity
Bells and whistles can be added to the above “basic” mean-reversion strategy
by incorporating volume and open interest ﬁlters. Let Vi be the total volume
(2007), de Roon et al. ( 1998, 2000), Dunis et al. ( 2006, 2010), Dutt et al. ( 1997), Frino and McKenzie
(2002), Girma and Paulson ( 1998), Hou and Nordén ( 2018), Kawaller et al. ( 2002), Kim and Leuthold
(1997), McComas ( 2003), Moore et al. ( 2006), Ng and Pirrong ( 1994), Perchanok ( 2012), Perchanok
and Kakabadse ( 2013), Poitras ( 1990), Ross ( 2006), Salcedo ( 2004), Schap ( 2005), Shimko ( 1994), Till
and Eagleeye ( 2017), van den Goorbergh ( 2004).
10For some additional pertinent literature, see, e.g., Bali and Demirtas ( 2008), Bessembinder et al. ( 1995),
Bianchi et al. ( 2015), Chaves and Viswanathan ( 2016), Fuertes et al. ( 2015), Irwin et al. ( 1996), Julio et
al. ( 2013), Leung et al. ( 2016), Monoyios and Sarno ( 2002), Rao ( 2011), Rosales and McMillan ( 2017),
Tse (2017).

10 Futures 169
for the futures labeled by i over the last week (i.e., the sum of daily volumes
over the last week), and V ′
i be the total volume over the prior week. Let Ui
and U ′
i be the analogous quantities for the open interest. Let
vi = ln(Vi /V ′
i )( 10.10)
ui = ln(Ui /U ′
i )( 10.11)
Then the strategy can be built, e.g., by taking the upper half of the futures by
the volume factor vi , taking the lower half of these futures by the open interest
factor ui , and applying the strategy deﬁned by Eq. ( 10.8) to this subset of the
futures.11
10.4 Strategy: Trend Following (Momentum)
Various momentum strategies for futures can be constructed similarly to those
for stocks. Here is a simple example (see, e.g., Balta and Kosowski 2013;
Moskowitz et al. 2012).12 Let Ri be the returns for the futures labeled by
i = 1,..., N over the past period T (which can be measured in, e.g., days,
weeks, or months). Then the weights wi of the trading portfolio are given by
wi = γ ηi
σi
(10.12)
ηi = sign(Ri )( 10.13)
where σi are the historical volatilities (computed over the period T or some
other period), and γ> 0 is ﬁxed via the normalization condition
N∑
i =1
|wi |= 1 (10.14)
Note that this strategy is equivalent to the optimization strategy (see Sect. 3.18,
Eq. 3.85) with a diagonal covariance matrix Cij = σ2
i δij (i.e., the correlations
11The rationale behind this is that: (i) larger volume changes are likely indicative of greater overreaction
(see, e.g., Bloom et al. 1994; Conrad et al. 2013; DeBondt and Thaler 1985; Gervais and Odean 2001;
Odean 2002; Statman et al. 2006), so a greater “snap-back” (i.e., mean-reversion) effect can be expected;
and (ii) open interest is related to trading by hedgers and is a proxy for market depth (see, e.g., Bessembinder
and Seguin 1993), so an increase in open interest is indicative of a deeper market where volume increases
have smaller effects on prices as compared with when there is a decrease in open interest.
12For some additional pertinent literature, see, e.g., Ahn et al. ( 2002), Bianchi et al. ( 2015), Dusak ( 1973),
F u e r t e se ta l .(2010, 2015), Hayes (2011), Kazemi and Li ( 2009), Miffre and Rallis ( 2007), Pirrong (2005),
Reynauld and T essier (1984), Schneeweis and Gupta ( 2006), Szakmary et al. ( 2010).

170 Z. Kakushadze and J. A. Serur
between different futures are ignored) and the expected returns Ei = ηi σi .
This is to be contrasted with the expected returns based on the cumulative
returns (Eq. 3.2), which in this case equal Ri . One issue with using Ei = ηi σi
as opposed to Ei = Ri is that, for small |Ri | (e.g., compared with σi ), ηi
can ﬂip even though the change in Ri is small. This results in an undesirable
instability in the strategy. There are ways to mitigate this, e.g., by smoothing
via ηi = tanh(Ri /κ),w h e r e κ is some parameter, e.g., the cross-sectional
standard deviation of Ri (see, e.g., Kakushadze 2015). Alternatively, one may
simply take Ei = Ri (and further use a non-diagonal Cij ). Also, note that the
weights deﬁned by Eq. ( 10.12) are not dollar-neutral. This can be rectiﬁed by
demeaning them:
wi = γ
⎡
⎣ηi
σi
− 1
N
N∑
j =1
ηj
σj
⎤
⎦ (10.15)
One shortcoming of this is that now some futures with ηi > 0 may be sold,
and some futures with ηi < 0 may be bought. T o avoid this, if the number
N+ =| H+| of the futures with ηi > 0 is not dramatically different from the
number N− =| H−| of the futures with ηi < 0 (here H± ={ i |± ηi > 0}),
we can take the weights to be
wi = γ+
ηi
σi
, i ∈ H+ (10.16)
wi = γ−
ηi
σi
, i ∈ H− (10.17)
So, now we have two parameters γ±, which can be ﬁxed to satisfy Eq. ( 10.14)
and the dollar-neutrality condition
N∑
i =1
wi = 0 (10.18)
However, if most ηi are positive (negative), i.e., we have skewed returns, then
long (short) positions will be well-diversiﬁed, while the short (long) positions
will not be. This can happen, e.g., if the broad market is rallying (selling off).
Equation (10.15) mitigates this to some extent. However, η
i can still be skewed
in this case. A simple way to avoid this altogether is to use the demeaned returns
˜Ri instead of Ri ,w h e r e ˜Ri = Ri − Rm , and the “market index” return Rm

10 Futures 171
is deﬁned by Eq. ( 10.7).13 Then ηi = sign(˜Ri ) are no longer skewed and
dollar-neutrality can be achieved as above. 14
References
Abken, P . A. (1989). An Analysis of Intra-market Spreads in Heating Oil Futures.
Journal of Futures Markets , 9(1), 77–86.
Adrangi, B., Chatrath, A., Song, F ., & Szidarovszky, F . (2006). Petroleum Spreads and
the T erm Structure of Futures Prices. Applied Economics, 38(16), 1917–1929.
Ahmadi, H. Z., Sharp, P . A., & Walther, C. H. (1986). The Effectiveness of Futures
and Options in Hedging Currency Risk. In F . Fabozzi (Ed.), Advances in Futures
and Options Research (Vol. 1, Part B, pp. 171–191). Greenwich, CT: JAI Press, Inc.
Ahn, D.-H., Boudoukh, J., Richardson, M., & Whitelaw, R. F . (2002). Partial Adjust-
ment or Stale Prices? Implications from Stock Index and Futures Return Autocor-
relations. Review of Financial Studies , 15(2), 655–689.
Anderson, R. W ., & Danthine, J. P . (1981). Cross Hedging. Journal of Political Econ-
omy, 89(6), 1182–1196.
Ankirchner, S., Dimitroff, G., Heyne, G., & Pigorsch, C. (2012). Futures Cross-
Hedging with a Stationary Basis. Journal of Financial and Quantitative Analysis ,
47 (6), 1361–1395.
Ankirchner, S., & Heyne, G. (2012). Cross Hedging with Stochastic Correlation.
Finance and Stochastics , 16 (1), 17–43.
Babbs, S. H., & Nowman, B. K. (1999). Kalman Filtering of Generalized Vasicek T erm
Structure Models. Journal of Financial and Quantitative Analysis , 34 (1), 115–130.
Baillie, R. T ., & Myers, R. J. (1991). Bivariate GARCH Estimation of the Optimal
Commodity Futures Hedge. Journal of Applied Econometrics , 6 (2), 109–124.
Bali, T . G., & Demirtas, K. O. (2008). T esting Mean Reversion in Financial Market
Volatility: Evidence from S&P 500 Index Futures. Journal of Futures Markets, 28(1),
1–33.
Balta, A.-N., & Kosowki, R. (2013). Momentum Strategies in Futures Markets and
T rend-Following Funds(Working Paper). Available online: https://www.edhec.edu/
sites/www.edhec-portail.pprod.net/ﬁles/publications/pdf/edhec-working-paper-
momentum-strategies-in-futures_1410350911195-pdfjpg .
Barrett, W . B., & Kolb, R. W . (1995). Analysis of Spreads in Agricultural Futures.
Journal of Futures Markets , 15(1), 69–86.
13I.e., in this case the momentum, winners and losers are deﬁned w.r.t. the market index, and the so-deﬁned
winners are bought, while the losers are sold.
14Further, instead of using cumulative returns Ri , one can use exponential moving averages (to sup-
press past contributions—see Chapter 3), the Hodrick-Prescott ﬁlter (to remove the noise and identify
the trend—see Chapter 8), the Kalman ﬁlter (see, e.g., Babbs and Nowman 1999; Benhamou 2016;
Bruder et al. 2013; DeMoura et al. 2016; Elliott et al. 2005; Engle and Watson 1987;H a r v e y 1984;
Harvey 1990;H a t e m i - Ja n dR o c a2006; Kalman 1960; Lautier and Galli 2004; Levine and Pedersen
2016; Martinelli and Rhoads 2010; Vidyamurthy 2004), or some other time-series ﬁlters.

172 Z. Kakushadze and J. A. Serur
Benet, B. A. (1990). Commodity Futures Cross Hedging of Foreign Exchange Expo-
sure. Journal of Futures Markets , 10(3), 287–306.
Benhamou, E. (2016). T rend Without Hiccups—A Kalman Filter Approach (Working
Paper). Available online: https://ssrn.com/abstract=2747102.
Bernstein, J. (1990). Jake Bernstein ’s Seasonal Futures Spreads: High-Probability Seasonal
Spreads for Futures T raders. Hoboken, NJ: Wiley.
Bessembinder, H. (1992). Systematic Risk, Hedging Pressure, and Risk Premiums in
Futures Markets. Review of Financial Studies , 5(4), 637–667.
Bessembinder, H. (1993). An Empirical Analysis of Risk Premia in Futures Markets.
Journal of Futures Markets , 13(6), 611–630.
Bessembinder, H., & Chan, K. (1992). Time-Varying Risk Premia and Forecastable
Returns in Futures Markets. Journal of Financial Economics , 32(2), 169–193.
Bessembinder, H., Coughenour, J. F ., Seguin, P . J., & Smoller, M. M. (1995). Mean
Reversion in Equilibrium Asset Prices: Evidence from the Futures T erm Structure.
Journal of Finance , 50(1), 361–375.
Bessembinder, H., & Seguin, P . J. (1993). Price Volatility, T rading Volume, and Mar-
ket Depth: Evidence from Futures Markets. Journal of Financial and Quantitative
Analysis, 28(1), 21–39.
Bianchi, R. J., Drew, M., & Fan, J. (2015). Combining Momentum with Reversal in
Commodity Futures. Journal of Banking & Finance , 59, 423–444.
Billingsley, R. S., & Chance, D. M. (1988). The Pricing and Performance of Stock
Index Futures Spreads. Journal of Futures Markets , 8(3), 303–318.
Blake, M. L., & Catlett, L. (1984). Cross Hedging Hay Using Corn Futures: An
Empirical T est.Western Journal of Agricultural Economics , 9(1), 127–134.
Blank, S. C. (1984). Cross Hedging Australian Cattle. Australian Journal of Agricultural
Economics, 28(2–3), 153–162.
Bloom, L., Easley, D., & O’Hara, M. (1994). Market Statistics and T echnical Analysis:
The Role of Volume. Journal of Finance , 49(1), 153–181.
Booth, J. R., Smith, R. L., & Stolz, R. W . (1984). The Use of Interest Rate Futures
by Financial Institutions. Journal of Bank Research , 15(1), 15–20.
Briys, E., & Solnik, B. (1992). Optimal Currency Hedge Ratios and Interest Rate
Risk. Journal of International Money and Finance , 11(5), 431–445.
Brooks, C., & Chong, J. (2001). The Cross-Currency Hedging Performance of
Implied Versus Statistical Forecasting Models. Journal of Futures Markets , 21(11),
1043–1069.
Brooks, C., Davies, R. J., & Kim, S. S. (2007). Cross Hedging with Single Stock
Futures. Assurances et Gestion des Risques , 74 (4), 473–504.
Brooks, C., Henry, O. T ., & Persand, G. (2002). The Effect of Asymmetries on
Optimal Hedge Ratios. Journal of Business , 75(2), 333–352.
Bruder, B., Dao, T .-L., Richard, R.-J., & Roncalli, T . (2013). T rend Filtering Meth-
ods for Momentum Strategies (Working Paper). Available online: https://ssrn.com/
abstract=2289097.
Castelino, M. G., & Vora, A. (1984). Spread Volatility in Commodity Futures: The
Length Effect. Journal of Futures Markets , 4 (1), 39–46.

10 Futures 173
Cecchetti, S. G., Cumby, R. E., & Figlewski, S. (1988). Estimation of the Optimal
Futures Hedge. Review of Economics and Statistics , 70(4), 623–630.
ˇCerovi´c, S., & Pepi´c, M. (2011). Interest Rate Derivatives in Developing Countries
in Europe. Perspectives of Innovation in Economics and Business , 9(3), 38–42.
Chang, J. S., & Fang, H. (1990). An Intertemporal Measure of Hedging Effectiveness.
Journal of Futures Markets , 10(3), 307–321.
Chaves, D. B., & Viswanathan, V . (2016). Momentum and Mean-Reversion in Com-
modity Spot and Futures Markets. Journal of Commodity Markets , 3(1), 39–53.
Chen, A. H., Kang, J., & Yang, B. (2005). A Model for Convexity-Based Cross-Hedges
with T reasury Futures. Journal of Fixed Income , 15(3), 68–79.
Chen, F ., & Sutcliffe, C. (2007). Better Cross Hedges With Composite Hedging?
Hedging Equity Portfolios Using Financial and Commodity Futures. European
Journal of Finance , 18(6), 575–595.
Cheung, C. W ., Kwan, C. C., & Yip, P . C. (1990). The Hedging Effectiveness of
Options and Futures: A Mean-Gini Approach. Journal of Futures Markets , 10(1),
61–73.
Clare, A. D., Ioannides, M., & Skinner, F . S. (2000). Hedging Corporate Bonds with
Stock Index Futures: A Word of Caution. Journal of Fixed Income , 10(2), 25–34.
Cole, C. A., Kastens, T . L., Hampel, F . A., & Gow, L. R. (1999). A Calendar Spread
T rading Simulation of Seasonal Processing Spreads. InProceedings of the NCCC-134
Conference on Applied Commodity Price Analysis, Forecasting, and Market Risk Man-
agement. Available online: http://www.farmdoc.illinois.edu/nccc134/conf_1999/
pdf/confp14-99.pdf .
Conrad, J., Dittmar, R. F ., & Ghysels, E. (2013). Ex Ante Skewness and Expected
Stock Returns. Journal of Finance , 68(1), 85–124.
Dahlgran, R. A. (2000). Cross-Hedging the Cottonseed Crush: A Case Study. Agribusi-
ness, 16 (2), 141–158.
Daigler, R. T . (2007). Spread Volume for Currency Futures. Journal of Economics and
Finance, 31(1), 12–19.
Daigler, R. T ., & Copper, M. (1998). A Futures Duration-Convexity Hedging
Method. Financial Review , 33(4), 61–80.
Davis, M. H. A. (2006). Optimal Hedging with Basis Risk. In Y. Kabanov, R. Liptser,
& J. Stoyanov (Eds.), From Stochastic Calculus to Mathematical Finance . Berlin,
Germany: Springer.
DeBondt, W . F . M., & Thaler, R. H. (1985). Does Stock Market Overreact? Journal
of Finance , 40(3), 793–807.
DeMaskey, A. L. (1997). Single and Multiple Portfolio Cross-Hedging with Currency
Futures. Multinational Finance Journal , 1(1), 23–46.
DeMaskey, A. L., & Pearce, J. A. (1998). Commodity and Currency Futures Cross-
Hedging of ASEAN Currency Exposures. Journal of T ransnational Management
Development, 4 (1), 5–24.
DeMoura, C. E., Pizzinga, A., & Zubelli, J. (2016). A Pairs T rading Strategy Based on
Linear State Space Models and the Kalman Filter. Quantitative Finance , 16 (10),
1559–1573.

174 Z. Kakushadze and J. A. Serur
de Roon, F . A., Nijman, T . E., & Veld, C. (1998). Pricing T erm Structure Risk in
Futures Markets. Journal of Financial and Quantitative Analysis , 33(1), 139–157.
de Roon, F . A., Nijman, T . E., & Veld, C. (2000). Hedging Pressure Effects in Futures
Markets. Journal of Finance , 55(3), 1437–1456.
Dunis, C., Laws, J., & Evans, B. (2006). T rading Futures Spreads. Applied Financial
Economics, 16 (12), 903–914.
Dunis, C., Laws, J., & Evans, B. (2010). T rading and Filtering Futures Spread Port-
folios. Journal of Derivatives & Hedge Funds , 15(4), 274–287.
Dusak, K. (1973). Futures T rading and Investor Returns: An Investigation of Com-
modity Market Risk Premiums. Journal of Political Economy , 81(6), 1387–1406.
Dutt, H. R., Fenton, J., Smith, J. D., & Wang, G. H. K. (1997). Crop Year Inﬂuences
and Variability of the Agricultural Futures Spreads. Journal of Futures Markets ,
17 (3), 341–367.
Ederington, L. H. (1979). The Hedging Performance of the New Futures Markets.
Journal of Finance , 34 (1), 157–170.
Elliott, R. J., van der Hoek, J., & Malcolm, W . P . (2005). Pairs T rading. Quantitative
Finance, 5(3), 271–276.
Engle, R. F ., & Watson, M. W . (1987). The Kalman Filter: Applications to Forecasting
and Rational-Expectation Models. In T . F . Bewley (Ed.), Fifth World Conference:
Advances in Econometrics (Vol. 1). Cambridge, UK: Cambridge University Press.
Falkenstein, E., & Hanweck, J. (1996). Minimizing Basis Risk from Non-parallel
Shifts in the Yield Curve. Journal of Fixed Income , 6 (1), 60–68.
Fisher, L., & Weil, R. L. (1971). Coping with the Risk of Interest-Rate Fluctuations:
Returns to Bondholders from Naïve and Optimal Strategies. Journal of Business ,
44 (4), 408–431.
Fortin, M., & Khoury, N. (1984). Hedging Interest Rate Risks with Financial Futures.
Canadian Journal of Administrative Sciences , 1(2), 367–382.
Foster, F . D., & Whiteman, C. H. (2002). Bayesian Cross Hedging: An Example from
the Soybean Market. Australian Journal of Management , 27 (2), 95–122.
Franken, J. R. V ., & Parcell, J. L. (2003). Cash Ethanol Cross-Hedging Opportunities.
Journal of Agricultural and Applied Economics , 35(3), 509–516.
Frino, A., & McKenzie, M. (2002). The Pricing of Stock Index Futures Spreads at
Contract Expiration. Journal of Futures Markets , 22(5), 451–469.
Fuertes, A., Miffre, J., & Fernandez-Perez, A. (2015). Commodity Strategies Based
on Momentum, T erm Structure, and Idiosyncratic Volatility. Journal of Futures
Markets, 35(3), 274–297.
Fuertes, A., Miffre, J., & Rallis, G. (2010). T actical Allocation in Commodity Futures
Markets: Combining Momentum and T erm Structure Signals. Journal of Banking
& Finance , 34 (10), 2530–2548.
Gay, G. D., & Kolb, R. W . (1983). The Management of Interest Rate Risk. Journal
of Portfolio Management , 9(2), 65–70.
Gay, G. D., Kolb, R. W ., & Chiang, R. (1983). Interest Rate Hedging: An Empirical
T est of Alternative Strategies. Journal of Financial Research , 6 (3), 187–197.

10 Futures 175
Géczy, C., Minton, B. A., & Schrand, C. (1997). Why Firms Use Currency Deriva-
tives. Journal of Finance , 52(4), 1323–1354.
Gervais, S., & Odean, T . (2001). Learning to Be Overconﬁdent. Review of Financial
Studies, 14 (1), 1–27.
Geske, R. L., & Pieptea, D. R. (1987). Controlling Interest Rate Risk and Return
with Futures. Review of Futures Markets , 6 (1), 64–86.
Ghosh, A. (1993). Hedging with Stock Index Futures: Estimation and Forecasting
with Error Correction Model. Journal of Futures Markets , 13(7), 743–752.
Girma, P . B., & Paulson, A. S. (1998). Seasonality in Petroleum Futures Spreads.
Journal of Futures Markets , 18(5), 581–598.
Grant, J. (2016). T rading Strategies in Futures Markets . Ph.D. thesis, Imperial Col-
lege, London, UK. Available online: https://spiral.imperial.ac.uk/bitstream/10044/
1/32011/1/Grant-J-2016-PhD-Thesis.PDFA.pdf .
Grieves, R., & Mann, S. V . (2004). An Overlooked Coupon Effect in T reasury Futures
Contracts. Journal of Derivatives , 12(2), 56–61.
Grieves, R., & Marcus, A. J. (2005). Delivery Options and T reasury-Bond Futures
Hedge Ratios. Journal of Derivatives , 13(2), 70–76.
Hanly, J., Morales, L., & Cassells, D. (2018). The Efﬁcacy of Financial Futures as a
Hedging T ool in Electricity Markets. International Journal of Financial Economics ,
23(1), 29–40.
Hartzog, J. (1982). Controlling Proﬁt Volatility: Hedging with GNMA Options.
Federal Home Loan Bank Board Journal , 15(2), 10–14.
Harvey, A. C. (1984). A Uniﬁed View of Statistical Forecasting Procedures. Journal of
Forecasting, 3(3), 245–275.
Harvey, A. C. (1990). Forecasting, Structural Time Series Models and the Kalman Filter .
Cambridge, UK: Cambridge University Press.
Hatemi-J, A., & Roca, E. (2006). Calculating the Optimal Hedge Ratio: Constant,
Time Varying and the Kalman Filter Approach. Applied Economics Letters , 13(5),
293–299.
Hayes, B. (2011). Multiple Time Scale Attribution for Commodity T rading Advisor
(CTA) Funds. Journal of Investment Management , 9(2), 35–72.
Hegde, S. P . (1982). The Impact of Interest Rate Level and Volatility on the Perfor-
mance of Interest Rate Hedges. Journal of Futures Markets , 2(4), 341–356.
Hilliard, J. E. (1984). Hedging Interest Rate Risk with Futures Portfolios under T erm
Structure Effects. Journal of Finance , 39(5), 1547–1569.
Hilliard, J., & Jordan, S. (1989). Hedging Interest Rate Risk with Futures Portfo-
lios Under Full-Rank Assumptions. Journal of Financial and Quantitative Analysis ,
24 (2), 217–240.
Holmes, P . (1996). Stock Index Futures Hedging: Hedge Ratio Estimation, Duration
Effects, Expiration Effects and Hedge Ratio Stability. Journal of Business Finance &
Accounting, 23(1), 63–77.
Ho, T ., & Saunders, A. (1983). Fixed Rate Loan Commitments, T ake-Down Risk,
and the Dynamics of Hedging with Futures. Journal of Financial and Quantitative
Analysis, 18(4), 499–516.

176 Z. Kakushadze and J. A. Serur
Hou, A. J., & Nordén, L. L. (2018). VIX Futures Calendar Spreads. Journal of Futures
Markets, 38(7), 822–838.
Hull, J. C. (2012). Options, Futures and Other Derivatives . Upper Saddle River, NJ:
Prentice Hall.
Irwin, S. H., Zulauf, C. R., & Jackson, T . E. (1996). Monte Carlo Analysis of Mean
Reversion in Commodity Futures Prices. American Journal of Agricultural Eco-
nomics, 78(2), 387–399.
Julio, I. F ., Hassan, M. K., & Ngene, G. M. (2013). T rading Strategies in Futures
Markets. Global Journal of Finance and Economics , 10(1), 1–12.
Kakushadze, Z. (2015). Mean-Reversion and Optimization. Journal of Asset Manage-
ment, 16 (1), 14–40. Available online: https://ssrn.com/abstract=2478345.
Kalman, P . E. (1960). A New Approach to Linear Filtering and Prediction Problems.
Journal of Basic Engineering , 82(1), 35–45.
Kawaller, I. G., Koch, P . D., & Ludan, L. (2002). Calendar Spreads, Outright Futures
Positions and Risk. Journal of Alternative Investments , 5(3), 59–74.
Kazemi, H., & Li, Y. (2009). Market Timing of CTAs: An Examination of Systematic
CTAs vs. Discretionary CTAs. Journal of Futures Markets , 29(11), 1067–1099.
Kim, M.-K., & Leuthold, R. M. (1997). The Distributional Behavior of Futures
Price Spread Changes: Parametric and Nonparametric T ests for Gold, T-Bonds, Corn,
and Live Cattle (Working Paper). Available online: https://ageconsearch.umn.edu/
bitstream/14767/1/aceo9703.pdf .
Kolb, R. W ., & Chiang, R. (1981). Improving Hedging Performance Using Interest
Rate Futures. Financial Management , 10(3), 72–79.
Kolb, R. W ., & Chiang, R. (1982). Duration, Immunization, and Hedging with
Interest Rate Futures. Journal of Financial Research , 5(2), 161–170.
Kroner, K. F ., & Sultan, J. (1993). Time-Varying Distributions and Dynamic Hedging
with Foreign Currency Futures. Journal of Financial and Quantitative Analysis ,
28(4), 535–551.
Kuberek, R. C., & Peﬂey, N. G. (1983). Hedging Corporate Debt with U.S. T reasury
Bond Futures. Journal of Futures Markets , 3(4), 345–353.
Lafuente, J. A. (2013). Optimal Cross-Hedging Under Futures Mispricing: A Note.
Journal of Derivatives & Hedge Funds , 19(3), 181–188.
Landes, W . J., Stoffels, J. D., & Seifert, J. A. (1985). An Empirical T est of a Duration-
Based Hedge: The Case of Corporate Bonds. Journal of Futures Markets , 5(2),
173–182.
Lautier, D., & Galli, A. (2004). Simple and Extended Kalman Filters: An Application
to T erm Structures of Commodity Prices. Applied Financial Economics , 14 (13),
963–973.
Lebeck, W . W . (1978). Futures T rading and Hedging. Food Policy,
3(1), 29–35.
Lee, S. B., & Oh, S. H. (1993). Managing Non-parallel Shift Risk of Yield Curve
with Interest Rate Futures. Journal of Futures Markets , 13(5), 515–526.
Leung, T ., Li, J., Li, X., & Wang, Z. (2016). Speculative Futures T rading Under Mean
Reversion. Asia-Paciﬁc Financial Markets , 23(4), 281–304.

10 Futures 177
Levine, A., & Pedersen, L. H. (2016). Which T rend Is Your Friend? Financial Analysts
Journal, 72(3), 51–66.
Lien, D. (1992). Optimal Hedging and Spreading in Cointegrated Markets. Economics
Letters, 40(1), 91–95.
Lien, D. (2004). Cointegration and the Optimal Hedge Ratio: The General Case.
Quarterly Review of Economics and Finance , 44 (5), 654–658.
Lien, D., & Luo, X. (1993). Estimating Multiperiod Hedge Ratios in Cointegrated
Markets. Journal of Futures Markets , 13(8), 909–920.
Lien, D., &Tse, Y. K. (2000). Hedging Downside Risk with Futures Contracts. Applied
Financial Economics , 10(2), 163–170.
Lindahl, M. (1992). Minimum Variance Hedge Ratios for Stock Index Futures: Dura-
tion and Expiration Effects. Journal of Futures Markets , 12(1), 33–53.
Low, A., Muthuswamy, J., Sakar, S., & T erry, E. (2002). Multiperiod Hedging with
Futures Contracts. Journal of Futures Markets , 22(12), 1179–1203.
Martinelli, R., & Rhoads, N. (2010). Predicting Market Data Using the Kalman Filter,
Part 1 and Part 2. T echnical Analysis of Stocks & Commodities , 28(1), 44–47. Ibid.,
28(2), 46–51.
McComas, A. (2003, July). Getting T echnical with Spreads. Futures Magazine ,p p .
52–55.
McEnally, R. W ., & Rice, M. L. (1979). Hedging Possibilities in the Flotation of Debt
Securities. Financial Management , 8(4), 12–18.
Miffre, J., & Rallis, G. (2007). Momentum Strategies in Commodity Futures Markets.
Journal of Banking & Finance , 31(6), 1863–1886.
Monoyios, M. (2004). Performance of Utility-Based Strategies for Hedging Basis Risk.
Quantitative Finance , 4 (3), 245–255.
Monoyios, M., & Sarno, L. (2002). Mean Reversion in Stock Index Futures Markets:
A Nonlinear Analysis. Journal of Futures Markets , 22(4), 285–314.
Moore, S., T oepke, J., & Colley, N. (2006). The Encyclopedia of Commodity and
Financial Spreads. Hoboken, NJ: Wiley.
Moosa, I. A. (2003). The Sensitivity of the Optimal Hedge Ratio to Model Speciﬁca-
tion. Finance Letters , 1(1), 15–20.
Moskowitz, T . J., Ooi, Y. H., & Pedersen, L. H. (2012). Time Series Momentum.
Journal of Financial Economics , 104 (2), 228–250.
Mun, K.-C. (2016). Hedging Bank Market Risk with Futures and Forwards. Quarterly
Review of Economics and Finance , 61, 112–125.
Mun, K.-C., & Morgan, G. E. (1997). Cross-Hedging Foreign Exchange Rate Risks:
The Case of Deposit Money Banks in Emerging Asian Countries. Paciﬁc-Basin
Finance Journal, 5(2), 215–230.
Myers, R. J. (1991). Estimating Time-Varying Optimal Hedge Ratios on Futures
Markets. Journal of Futures Markets , 11(1), 39–53.
Ng, V . K., & Pirrong, S. C. (1994). Fundamentals and Volatility: Storage, Spreads,
and the Dynamics of Metals Prices. Journal of Business , 67 (2), 203–230.
Odean, T . (2002). Volume, Volatility, Price, and Proﬁt When All T raders Are Above
Average. Journal of Finance , 53(6), 1887–1934.

178 Z. Kakushadze and J. A. Serur
Pepi´c, M. (2014). Managing Interest Rate Risk with Interest Rate Futures. Ekonomika
preduze´ca, 62(3–4), 201–209.
Perchanok, K. (2012). Futures Spreads: Theory and Praxis . Ph.D. thesis, The University
of Northampton, Northampton, UK. Available online: http://nectar.northampton.
ac.uk/4963/1/Perchanok20124963.pdf.
Perchanok, K., & Kakabadse, N. (2013). Causes of Market Anomalies of Crude
Oil Calendar Spreads: Does Theory of Storage Address the Issue? Problems and
Perspectives in Management , 11(2), 35–47.
Picou, G. (1981, May–June). Managing Interest Rate Risk with Interest Rate Futures.
Bankers Magazine , Vol. 164, pp. 76–81.
Pirrong, C. (2005). Momentum in Futures Markets (Working Paper). Available online:
https://ssrn.com/abstract=671841.
Pitts, M. (1985).The Management of Interest Rate Risk: Comment. Journal of Portfolio
Management, 11(4), 67–69.
Poitras, G. (1990). The Distribution of Gold Futures Spreads. Journal of Futures
Markets, 10(6), 643–659.
Rao, V . K. (2011). Multiperiod Hedging Using Futures: Mean Reversion and the
Optimal Hedging Path. Journal of Risk and Financial Management , 4 (1), 133–
161.
Rendleman, R. J. (1999). Duration-Based Hedging with T reasury Bond Futures. Jour-
nal of Fixed Income , 9(1), 84–91.
Reynauld, J., & T essier, J. (1984). Risk Premiums in Futures Markets: An Empirical
Investigation. Journal of Futures Markets , 4 (2), 189–211.
Rosales, E. B., & McMillan, D. (2017). Time-Series and Cross-Sectional Momen-
tum and Contrarian Strategies Within the Commodity Futures Markets. Cogent
Economics & Finance , 5(1), 1339772.
Ross, J. (2006, December). Exploiting Spread T rades. Futures Magazine, pp. 34–36.
Salcedo, Y. (2004, September). Spreads for the Fall. Futures Magazine, pp. 54–57.
Schap, K. (2005). The Complete Guide to Spread T rading.N e wY o r k ,N Y :M c G r a w - H i l l .
Schneeweis, T ., & Gupta, R. (2006). Diversiﬁcation Beneﬁts of Managed Futures.
Journal of Investment Consulting , 8(1), 53–62.
Shimko, D. C. (1994). Options on Futures Spreads: Hedging, Speculation, and Val-
uation. Journal of Futures Markets , 14 (2), 183–213.
Statman, M., Thorley, S., & Vorkink, K. (2006). Investor Overconﬁdence and T rading
Volume. Review of Financial Studies , 19(4), 1531–1565.
Szakmary, A. C., Shen, Q., & Sharma, S. C. (2010). T rend-Following T rading Strate-
gies in Commodity Futures: A Re-examination. Journal of Banking & Finance ,
34 (2), 409–426.
Till, H., & Eagleeye, J. (2017). Commodity Futures T rading Strategies: T rend-
Following and Calendar Spreads (Working Paper). Available online: https://ssrn.
com/abstract=2942340.
T oevs, A., & Jacob, D. (1986). Futures and Alternative Hedge Ratio Methodologies.
Journal of Portfolio Management , 12(3), 60–70.

10 Futures 179
T rainer, F . H., Jr. (1983). The Uses of T reasury Bond Futures in Fixed Income Portfolio
Management. Financial Analysts Journal , 39(1), 27–34.
Tse, Y. (2017). Return Predictability and Contrarian Proﬁts of International Index
Futures. Journal of Futures Markets , 38(7), 788–803.
van den Goorbergh, R. W . J. (2004). Essays on Optimal Hedging and Investment Strate-
gies and on Derivative Pricing. Ph.D. thesis,Tilburg University,Tilburg,The Nether-
lands.
Vidyamurthy, G. (2004). Pairs T rading: Quantitative Methods and Analysis . Hoboken,
NJ: Wiley.
Viswanath, P . V . (1993). Efﬁcient Use of Information, Convergence Adjustments, and
Regression Estimates of Hedge Ratios. Journal of Futures Markets , 13(1), 43–53.
Wang, C., & Yu, M. (2004). T rading Activity and Price Reversals in Futures Markets.
Journal of Banking & Finance , 28(6), 1337–1361.
Wolf, A. (1987). Optimal Hedging with Futures Options. Journal of Economics and
Business, 39(2), 141–158.
Working, H. (1953). Futures T rading and Hedging.American Economic Review, 43(3),
314–434.
Yawitz, J. B., & Marshall, W . B. (1985). The Use of Futures in Immunized Portfolios.
Journal of Portfolio Management , 11(2), 51–55.
Yeutter, C., & Dew, J. K. (1982).The Use of Futures in Bank Loans. In H. V . Prochnow
(Ed.), Bank Credit.N e wY o r k ,N Y :H a r p e ra n dR o w .
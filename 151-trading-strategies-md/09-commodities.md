# Chapter 9: Commodities

9
Commodities
9.1 Strategy: Roll Yields
When commodity futures are in backwardation (contango), i.e., when the term
structure of futures prices is downward (upward) sloping, long (short) futures
positions on average generate positive returns due to the roll yield. Roll yields
come from rebalancing futures positions: when the current long (short) futures
contract is about to expire, it is sold (covered) and another futures contract
with longer expiration is bought (sold). Let
φ = P
1/P2 (9.1)
where P1 is the front-month futures price, and P2 is the second-month futures
price. The ratio φ is a measure of backwardation ( φ> 1) and contango
(φ< 1). A zero-cost long-short portfolio can then be built based on φ, e.g.,
by buying commodity futures with higher values of φ and selling futures with
lower values thereof. 1
1For some pertinent literature, see, e.g., Anson ( 1998), Arnott et al. ( 2014), Erb and Harvey ( 2006), Fama
and French ( 1987, 1988), Feldman and Till ( 2006), Fuertes et al. ( 2015), Gorton et al. ( 2013), Gorton
and Rouwenhorst ( 2006), Greer ( 2000), Leung et al. ( 2016), Ma et al. ( 1992), Mou ( 2010), Mouakhar
and Roberge ( 2010), Symeonidis et al. ( 2012), T aylor (2016), T elser (1958).
© The Author(s) 2018
Z. Kakushadze and J. A. Serur, 151 T rading Strategies,
https://doi.org/10.1007/978-3-030-02792-6_9
155

156 Z. Kakushadze and J. A. Serur
9.2 Strategy: T rading Based on Hedging
Pressure
This strategy is based on hedgers’ and speculators’ position data provided
(weekly) by the U.S. Commodity Futures T rading Commission (CFTC) in
the Commitments of T raders (COT) reports. For each commodity, the “hedg-
ing pressure” (HP), separately for hedgers and speculators, is calculated as the
number of long contracts divided by the total number of contracts (long plus
short). So, HP is between 0 and 1. High (low) hedgers’ HP is indicative of
contango (backwardation), while high (low) speculators’ HP is indicative of
backwardation (contango). A zero-cost portfolio can be constructed, e.g., as
follows. First, the cross-section of commodities is divided into the upper half
and the lower half by the speculators’ HP . Then, the upper half commodity
futures are bought if they are in the bottom quintile by the hedgers’ HP , and
the lower half commodity futures are sold if they are in the top quintile by the
hedger’s HP . T ypical formation and holding periods are 6 months.
2
9.3 Strategy: Portfolio Diversification with
Commodities
Commodity markets typically have a low correlation with equity markets,
which can be used to improve performance characteristics of equity portfolios
by combining equity and commodity investments. There are different ways to
do this. A “passive approach” would amount to buying commodities with a
preset portion of the available funds, holding them, and rebalancing the port-
folio with some periodicity (e.g., monthly or annually). An “active approach”
would amount to a tactical asset allocation approach via increasing/decreasing
the exposure to commodities based on an increase/decrease in the Fed discount
rate (empirically, commodity returns tend to be sizably correlated with the Fed
monetary policy) or some other methodology.
3
2For some literature on trading strategies based on such data and related topics, see, e.g., Basu and Miffre
(2013), Bessembinder ( 1992), Carter et al. ( 1983), Cheng and Xiong ( 2013), de Roon et al. ( 2000),
Dewally et al. ( 2013), Fernandez-Perez et al. ( 2016), Fishe et al. ( 2014), Fuertes et al. ( 2015), Hirshleifer
(1990), Lehecka ( 2013), Miffre ( 2012), Switzer and Jiang ( 2010).
3For some literature on diversiﬁcation strategies using commodities and related topics, see, e.g., Adams
and Glück ( 2015), Bernardi et al. ( 2018), Bjornson and Carter ( 1997), Blitz and Van Vliet ( 2008), Bodie
(1983), Bodie and Rosansky ( 1980), Chan et al. ( 2011), Chance ( 1994), Chong and Miffre ( 2010),
Conover et al. ( 2010), Creti et al. ( 2013), Daumas ( 2017), Draper et al. ( 2006), Edwards and Park ( 1996),
Elton et al. ( 1987), Frankel ( 2006), Gorton and Rouwenhorst ( 2006), Greer ( 1978), Greer ( 2007), Hess
et al. (2008), Jensen et al. ( 2000, 2002), Kaplan and Lummer ( 1998), Lummer and Siegel ( 1993), Marshall

9 Commodities 157
9.4 Strategy: Value
This strategy is similar to the value strategy for stocks (see Sect. 3.3). Value for
commodities can be deﬁned as, e.g., the ratio (see, e.g., Asness et al. 2013)
v = P5/P0 (9.2)
where P5 is the spot price 5 years ago, 4 and P0 is the current spot price. Then
one can build a zero-cost portfolio by, e.g., buying the commodities in the
top tercile by value, and selling those in the bottom tercile. The portfolio is
rebalanced monthly.
9.5 Strategy: Skewness Premium
This strategy is based on the empirically observed negative correlation between
the skewness of historical returns and future expected returns of the commodity
futures. The skewness Si is deﬁned as ( i = 1,..., N labels different commodi-
ties):
Si = 1
σ3
i T
T∑
s=1
[
Ris − Ri
]3 (9.3)
Ri = 1
T
T∑
s=1
Ris (9.4)
σ2
i = 1
T − 1
T∑
s=1
[
Ris − Ri
]2 (9.5)
where Ris are the time series of historical returns (with T observations in each
time series). A zero-cost strategy can be built by, e.g., buying the commodity
futures in the bottom quintile by skewness, and selling the futures in the top
quintile.5
et al. ( 2008), Miffre and Rallis ( 2007), Nguyen and Sercu ( 2010), T aylor ( 2004) ,V r u g te ta l .( 2007),
Wang and Yu ( 2004), Weiser ( 2003).
4Or the average spot price between 5.5 and 4.5 years ago.
5See, e.g., Fernandez-Perez et al. ( 2018). For some additional pertinent literature, see, e.g., Barberis and
Huang ( 2008), Christie-David and Chaudry ( 2001), Eastman and Lucey ( 2008), Gilbert et al. ( 2006),
Junkus ( 1991), Kumar ( 2009), Lien ( 2010), Lien and Wang ( 2015), Mitton and Vorkink ( 2007), Stulz
(1996), Tversky and Kahneman ( 1992).

158 Z. Kakushadze and J. A. Serur
9.6 Strategy: T rading with Pricing Models
Commodity futures term structure is nontrivial. One way to model it is via
stochastic processes. Let S(t ) be the spot price, and let X (t ) = ln(S(t )).T h e n
X (t ) can be modeled using, e.g., a mean-reverting Brownian motion (i.e., the
Ornstein-Uhlenbeck process [Uhlenbeck and Ornstein 1930])6:
dX (t ) = κ [a − X (t )] dt + σ dW (t )( 9.6)
Here the parameters κ (mean-reversion parameter), a (the long-run mean)
and σ (log-volatility) are assumed to be constant; and W (t ) is a Q-Brownian
motion, where Q is a risk-free probability measure. 7 The standard claim pricing
argument (see, e.g., Baxter and Rennie 1996;H u l l 2012; Kakushadze 2015)
gives for the futures price F (t, T ) (which is the price at time t of the futures
contract with the delivery date T )
F (t, T ) = Et (S(T )) ( 9.7)
ln(F (t, T )) = Et (X (T )) + 1
2 Vt (X (T )) ( 9.8)
Here Et (·) and Vt (·) are the conditional expectation and variance, respectively,
at time t . This gives:
ln(F (t, T )) = exp (−κ(T − t )) X (t ) + a
[
1 − exp (−κ(T − t ))
]
+
+ σ2
4κ
[
1 − exp (−2κ(T − t ))
]
(9.9)
The parameters κ, a,σ can be ﬁtted using historical data (e.g., using nonlinear
least squares). Then the current market price can be compared to the model
price to identify the futures that are rich (sell signal) and cheap (buy signal)
compared with the model prediction. Here two cautionary remarks are in order.
6This is a one-factor model. More complex models including multifactor models, nonconstant/stochastic
volatility models, etc., can be considered instead. For some literature on modeling futures prices via
stochastic processes and related topics, see, e.g., Andersen ( 2010), Bessembinder et al. ( 1995), Borovkova
and Geman ( 2006), Casassus and Collin-Dufresne ( 2005), Chaiyapo and Phewchean ( 2017), Choi et al.
(2014), Geman and Roncoroni ( 2006), Gibson and Schwartz ( 1990), Hilliard and Reis ( 1998), Jankow-
itsch and Nettekoven ( 2008), Litzenberger and Rabinowitz ( 1995), Liu and T ang (2011), Milonas ( 1991),
Miltersen and Schwartz ( 1998), Ng and Pirrong ( 1994), Nielsen and Schwartz ( 2004), Paschke and
Prokopczuk ( 2012), Pindyck ( 2001), Routledge et al. ( 2000), Schwartz ( 1997, 1998), Schwartz and
Smith (2000).
7Note that this model reduces to the Black-Scholes model (Black and Scholes 1973)i nt h el i m i t κ → 0,
a →∞ , κ a = ﬁxed.

9 Commodities 159
First, the model ﬁt could work in-sample but have no predictive power out-of-
sample, so the forecasting power needs to be ascertained (see, e.g., Paschke and
Prokopczuk 2012). Second, a priori we could write down any reasonable term
structure model with desirable qualitative properties (e.g., mean-reversion) and
ﬁt the parameters using historical data without any reference to an underlying
stochastic dynamics whatsoever, including using, e.g., “black-box” machine
learning techniques. So long as the model works out-of-sample, there is no
magic bullet here and “fancy” does not equal “better”.
References
Adams, Z., & Glück, T . (2015). Financialization in Commodity Markets: A Passing
T rend or the New Normal? Journal of Banking & Finance , 60, 93–111.
Andersen, L. B. G. (2010). Markov Models for Commodity Futures: Theory and
Practice. Quantitative Finance , 10 (8), 831–854.
Anson, M. J. P . (1998). Spot Returns, Roll Yield, and Diversiﬁcation with Commodity
Futures. Journal of Alternative Investments , 1(3), 16–32.
Arnott, R., Chaves, D., Gunzberg, J., Hsu, J., &Tsui, P . (2014, November/December).
Getting Smarter About Commodities: An Index to Counter the Possible Pitfalls.
Journal of Indexes , 2014, 52–60.
Asness, C., Moskowitz, T ., & Pedersen, L. H. (2013). Value and Momentum Every-
where. Journal of Finance , 68(3), 929–985.
Barberis, N., & Huang, M. (2008). Stocks as Lotteries: The Implications of Probability
Weighting for Security Prices. American Economic Review , 98(5), 2066–2100.
Basu, D., & Miffre, J. (2013). Capturing the Risk Premium of Commodity Futures:
The Role of Hedging Pressure. Journal of Banking & Finance , 37 (7), 2652–2664.
Baxter, M., & Rennie, A. (1996). Financial Calculus: An Introduction to Derivative
Pricing. Cambridge, UK: Cambridge University Press.
Bernardi, S., Leippold, M., & Lohre, H. (2018). Maximum Diversiﬁcation Strategies
along Commodity Risk Factors. European Financial Management , 24 (1), 53–78.
Bessembinder, H. (1992). Systematic Risk, Hedging Pressure, and Risk Premiums in
Futures Markets. Review of Financial Studies , 5 (4), 637–667.
Bessembinder, H., Coughenour, J. F ., Seguin, P . J., & Smoller, M. M. (1995). Mean
Reversion in Equilibrium Asset Prices: Evidence from the Futures T erm Structure.
Journal of Finance , 50 (1), 361–375.
Bjornson, B., & Carter, C. A. (1997). New Evidence on Agricultural Commodity
Return Performance Under Time-Varying Risk. American Journal of Agricultural
Economics, 79 (3), 918–930.
Black, F ., & Scholes, M. (1973). The Pricing of Options and Corporate Liabilities.
Journal of Political Economy , 81(3), 637–659.

160 Z. Kakushadze and J. A. Serur
Blitz, D., & Van Vliet, P . (2008). Global T actical Cross Asset Allocation: Applying
Value and Momentum Across Asset Classes. Journal of Portfolio Management , 35 (1),
23–28.
Bodie, Z. (1983). Commodity Futures as a Hedge Against Inﬂation. Journal of Portfolio
Management, 9 (3), 12–17.
Bodie, Z., & Rosansky, V . I. (1980). Risk and Return in Commodity Futures. Financial
Analysts Journal , 36 (3), 27–39.
Borovkova, S., & Geman, H. (2006). Seasonal and Stochastic Effects in Commodity
Forward Curves. Review of Derivatives Research , 9 (2), 167–186.
Carter, C., Rausser, G., & Schmitz, A. (1983). Efﬁcient Asset Portfolios and the
Theory of Normal Backwardation. Journal of Political Economy , 91(2), 319–331.
Casassus, J., & Collin-Dufresne, P . (2005). Stochastic Convenience Yield Implied from
Commodity Futures and Interest Rates. Journal of Finance , 60 (5), 2283–2331.
Chaiyapo, N., & Phewchean, N. (2017). An Application of Ornstein-Uhlenbeck
Process to Commodity Pricing in Thailand. Advances in Difference Equations , 2017,
179.
Chan, K. F ., T reepongkaruna, S., Brooks, R., & Gray, S. (2011). Asset Market Link-
ages: Evidence from Financial, Commodity and Real Estate Assets. Journal of Bank-
ing & Finance , 35 (6), 1415–1426.
Chance, D. (1994). Managed Futures and Their Role in Investment Portfolios . Char-
lottesville, VA: The Research Foundation of the Institute of Chartered Financial
Analysts.
Cheng, I.-H., & Xiong, W . (2013). Why Do Hedgers T rade so Much? (Working Paper).
Available online: https://ssrn.com/abstract=2358762.
Choi, H. I., Kwon, S.-H., Kim, J. Y., & Jung, D.-S. (2014). Commodity Futures T erm
Structure Model. Bulletin of the Korean Mathematical Society , 51(6), 1791–1804.
Chong, J., & Miffre, J. (2010). Conditional Correlation and Volatility in Commodity
Futures and T raditional Asset Markets. Journal of Alternative Investments , 12(3),
61–75.
Christie-David, R., & Chaudry, M. (2001). Coskewness and Cokurtosis in Futures
Markets. Journal of Empirical Finance , 8(1), 55–81.
Conover, C. M., Jensen, G. R., Johnson, R. R., & Mercer, J. M. (2010). Is Now the
Time to Add Commodities to Your Portfolio? Journal of Investing , 19 (3), 10–19.
Creti, A., Joëts, M., & Mignon, V . (2013). On the Links Between Stock and Com-
modity Markets’ Volatility.Energy Economics , 37, 16–28.
Daumas, L. D. (2017). Hedging Stocks Through Commodity Indexes: A DCC-GARCH
Approach (Working Paper). Available online: https://impa.br/wp-content/uploads/
2017/11/RiO2017-PP_FAiube.pdf .
de Roon, F . A., Nijman, T . E., & Veld, C. (2000). Hedging Pressure Effects in Futures
Markets. Journal of Finance , 55 (3), 1437–1456.
Dewally, M., Ederington, L. H., & Fernando, C. S. (2013). Determinants of T rader
Proﬁts in Commodity Futures Markets. Review of Financial Studies , 26 (10), 2648–
2683.

9 Commodities 161
Draper, P ., Faff, R. W ., & Hillier, D. (2006). Do Precious Metals Shine? An Investment
Perspective. Financial Analysts Journal , 62(2), 98–106.
Eastman, A. M., & Lucey, B. M. (2008). Skewness and Asymmetry in Futures Returns
and Volumes. Applied Financial Economics , 18(10), 777–800.
Edwards, F . R., & Park, J. M. (1996). Do Managed Futures Make Good Investments?
Journal of Futures Markets , 16 (5), 475–517.
Elton, E. J., Gruber, M. J., & Rentzler, J. C. (1987). Professionally Managed, Publicly
T raded Commodity Funds. Journal of Business , 60 (2), 175–199.
Erb, C., & Harvey, C. (2006). The Strategic and T actical Value of Commodity Futures.
Financial Analysts Journal , 62(2), 69–97.
Fama, E. F ., & French, K. R. (1987). Commodity Futures Prices: Some Evidence on
Forecast Power, Premiums, and the Theory of Storage. Journal of Business , 60 (1),
55–73.
Fama, E. F ., & French, K. R. (1988). Business Cycles and the Behavior of Metals
Prices. Journal of Finance , 43(5), 1075–1093.
Feldman, B., &Till, H. (2006). Backwardation and Commodity Futures Performance:
Evidence from Evolving Agricultural Markets. Journal of Alternative Investments ,
9 (3), 24–39.
Fernandez-Perez, A., Fuertes, A. M., & Miffre, J. (2016). Is Idiosyncratic Volatility
Priced in Commodity Futures Markets? International Review of Financial Analysis ,
46, 219–226.
Fernandez-Perez, A., Frijns, B., Fuertes, A. M., & Miffre, J. (2018). The Skewness of
Commodity Futures Returns. Journal of Banking & Finance , 86, 143–158.
Fishe, R. P . H., Janzen, J. P ., & Smith, A. (2014). Hedging and Speculative T rading in
Agricultural Futures Markets. American Journal of Agricultural Economics , 96 (2),
542–556.
Frankel, J. A. (2006). The Effect of Monetary Policy on Real Commodity Prices. In
J. Campbell (Ed.), Asset Prices and Monetary Policy (pp. 291–333). Chicago, IL:
University of Chicago Press.
Fuertes, A., Miffre, J., & Fernandez-Perez, A. (2015). Commodity Strategies Based
on Momentum, T erm Structure, and Idiosyncratic Volatility. Journal of Futures
Markets, 35 (3), 274–297.
Geman, H., & Roncoroni, A. (2006). Understanding the Fine Structure of Electricity
Prices. Journal of Business , 79 (3), 1225–1261.
Gibson, R., & Schwartz, E. S. (1990). Stochastic Convenience Yield and the Pricing
of Oil Contingent Claims. Journal of Finance , 15 (3), 959–967.
Gilbert, S., Jones, S. K., & Morris, G. H. (2006). The Impact of Skewness in the
Hedging Decision. Journal of Futures Markets , 26 (5), 503–520.
Gorton, G. B., & Rouwenhorst, K. G. (2006). Facts and Fantasies About Commodity
Futures. Financial Analysts Journal , 62(2), 47–68.
Gorton, G. B., Hayashi, F ., & Rouwenhorst, K. G. (2013). The Fundamentals of
Commodity Futures Returns. Review of Finance , 17 (1), 35–105.
Greer, R. J. (1978). Conservative Commodities: A Key Inﬂation Hedge. Journal of
Portfolio Management , 4 (4), 26–29.

162 Z. Kakushadze and J. A. Serur
Greer, R. J. (2000). The Nature of Commodity Index Returns. Journal of Alternative
Investments, 3(1), 45–52.
Greer, R. J. (2007). The Role of Commodities in Investment Portfolios. CFA Institute
Conference Proceedings Quarterly , 24 (4), 35–44.
Hess, D., Huang, H., & Niessen, A. (2008). How Do Commodity Futures Respond
to Macroeconomic News? Financial Markets and Portfolio Management , 22(2),
127–146.
Hilliard, J., & Reis, J. (1998). Valuation of Commodity Futures and Options Under
Stochastic Convenience Yields, Interest Rates, and Jump Diffusions on the Spot.
Journal of Financial and Quantitative Analysis , 33(1), 61–86.
Hirshleifer, D. (1990). Hedging Pressure and Futures Price Movements in a General
Equilibrium Model. Econometrica, 58(2), 411–428.
Hull, J. C. (2012). Options, Futures and Other Derivatives . Upper Saddle River, NJ:
Prentice Hall.
Jankowitsch, R., & Nettekoven, M. (2008). T rading Strategies Based on T erm Struc-
ture Model Residuals. European Journal of Finance , 14 (4), 281–298.
Jensen, G. R., Johnson, R. R., & Mercer, J. M. (2000). Efﬁcient Use of Commodity
Futures in Diversiﬁed Portfolios. Journal of Futures Markets , 20 (5), 489–506.
Jensen, G. R., Johnson, R. R., & Mercer, J. M. (2002). T actical Asset Allocation and
Commodity Futures. Journal of Portfolio Management , 28(4), 100–111.
Junkus, J. C. (1991). Systematic Skewness in Futures Contracts. Journal of Futures
Markets, 11(1), 9–24.
Kakushadze, Z. (2015). Phynance. Universal Journal of Physics and Application , 9(2),
64–133. Available online: https://ssrn.com/abstract=2433826.
Kaplan, P ., & Lummer, S. L. (1998). Update: GSCI Collateralized Futures as a Hedg-
ing Diversiﬁcation T ool for Institutional Portfolios. Journal of Investing , 7 (4), 11–
18.
Kumar, A. (2009). Who Gambles in the Stock Market? Journal of Finance , 64 (4),
1889–1933.
Lehecka, G. V . (2013). Hedging and Speculative Pressures: An Investigation of the
Relationships among T rading Positions and Prices in Commodity Futures Markets.
In Proceedings of the NCCC-134 Conference on Applied Commodity Price Analysis,
Forecasting, and Market Risk Management . Available online: http://www.farmdoc.
illinois.edu/nccc134/conf_2013/pdf/Lehecka_NCCC-134_2013.pdf .
Leung, T ., Li, J., Li, X., & Wang, Z. (2016). Speculative Futures T rading Under Mean
Reversion. Asia-Paciﬁc Financial Markets , 23(4), 281–304.
Lien, D. (2010). The Effects of Skewness on Optimal Production and Hedging Deci-
sions: An Application of the Skew-Normal Distribution. Journal of Futures Markets ,
30 (3), 278–289.
Lien, D., & Wang, Y. (2015). Effects of Skewness and Kurtosis on Production
and Hedging Decisions: A Skewed t
Distribution Approach. European Journal
of Finance , 21(13–14), 1132–1143.
Litzenberger, R. H., & Rabinowitz, N. (1995). Backwardation in Oil Futures Markets:
Theory and Empirical Evidence. Journal of Finance , 50 (3), 1517–1545.

9 Commodities 163
Liu, P ., & T ang, K. (2011). The Stochastic Behavior of Commodity Prices with Het-
eroscedasticity in the Convenience Yield. Journal of Empirical Finance , 18(2), 211–
224.
Lummer, S. L., & Siegel, L. B. (1993). GSCI Collateralized Futures: A Hedging and
Diversiﬁcation T ool for Institutional Portfolio. Journal of Investing , 2(2), 75–82.
Ma, K., Mercer, M., & Walker, M. (1992). Rolling over Futures Contracts: A Note.
Journal of Futures Markets , 12(2), 203–217.
Marshall, B. R., Cahan, R. H., & Cahan, J. M. (2008). Can Commodity Futures Be
Proﬁtably T raded with Quantitative Market Timing Strategies? Journal of Banking
& Finance , 32(9), 1810–1819.
Miffre, J. (2012, January). Hedging Pressure-Based Long/Short Commodity Strategy
Used for Third Generation Commodity Index. Risk. Available online: https://www.
risk.net/2247251.
Miffre, J., & Rallis, G. (2007). Momentum Strategies in Commodity Futures Markets.
Journal of Banking & Finance , 31(6), 1863–1886.
Milonas, N. T . (1991). Measuring Seasonalities in Commodity Markets and the Half-
Month Effect. Journal of Futures Markets , 11(3), 331–346.
Miltersen, K. R., & Schwartz, E. S. (1998). Pricing of Options on Commodity Futures
with Stochastic T erm Structures of Convenience Yield and Interest Rates. Journal
of Financial and Quantitative Analysis , 33(1), 33–59.
Mitton, T ., & Vorkink, K. (2007). Equilibrium Underdiversiﬁcation and the Prefer-
ence for Skewness. Review of Financial Studies , 20 (4), 1255–1288.
Mou, Y. (2010). Limits to Arbitrage and Commodity Index Investment: Front-
Running the Goldman Roll (Working Paper). Available online: https://ssrn.com/
abstract=1716841.
Mouakhar, T ., & Roberge, M. (2010). The Optimal Approach to Futures Contract
Roll in Commodity Portfolios. Journal of Alternative Investments , 12(3), 51–60.
Ng, V . K., & Pirrong, S. C. (1994). Fundamentals and Volatility: Storage, Spreads,
and the Dynamics of Metals Prices. Journal of Business , 67 (2), 203–230.
Nguyen, V . T . T ., & Sercu, P . (2010).T actical Asset Allocation with Commodity Futures:
Implications of Business Cycle and Monetary Policy (Working Paper). Available online:
https://ssrn.com/abstract=1695889.
Nielsen, M. J., & Schwartz, E. S. (2004). Theory of Storage and the Pricing of Com-
modity Claims. Review of Derivatives Research , 7 (1), 5–24.
Paschke, R., & Prokopczuk, M. (2012). Investing in Commodity Futures Markets:
Can Pricing Models Help? European Journal of Finance , 18(1), 59–87.
Pindyck, R. S. (2001). The Dynamics of Commodity Spot and Futures Markets: A
Primer. Energy Journal , 22(3), 1–30.
Routledge, B., Seppi, D. J., & Spatt, C. (2000). Equilibrium Forward Curves for
Commodities. Journal of Finance , 55 (3), 1297–1338.
Schwartz, E. S. (1997). The Stochastic Behavior of Commodity Prices: Implications
for Valuation and Hedging. Journal of Finance , 52(3), 923–973.
Schwartz, E. S. (1998). Valuing Long-T erm Commodity Assets. Journal of Energy
Finance & Development , 3(2), 85–99.

164 Z. Kakushadze and J. A. Serur
Schwartz, E. S., & Smith, J. E. (2000). Short-T erm Variations and Long-term Dynam-
ics in Commodity Prices. Management Science , 46 (7), 893–911.
Stulz, R. M. (1996). Rethinking Risk Management. Journal of Applied Corporate
Finance, 9 (3), 8–25.
Switzer, L. N., Jiang, H. (2010). Market Efﬁciency and the Risks and Returns of
Dynamic T rading Strategies with Commodity Futures. In H. E. Stanley (Ed.),
Proceedings of the First Interdisciplinary Chess Interactions Conference (pp. 127–156).
Singapore: World Scientiﬁc Publishing.
Symeonidis, L., Prokopczuk, M., Brooks, C., & Lazar, E. (2012). Futures Basis, Inven-
tory and Commodity Price Volatility: An Empirical Analysis. Economic Modelling ,
29 (6), 2651–2663.
T aylor, N. (2004). Modeling Discontinuous Periodic Conditional Volatility: Evidence
from the Commodity Futures Market. Journal of Futures Markets , 24 (9), 805–834.
T aylor, N. (2016). Roll Strategy Efﬁciency in Commodity Futures Markets. Journal
of Commodity Markets , 1(1), 14–34.
T elser, L. G. (1958). Futures T rading and the Storage of Cotton and Wheat. Journal
of Political Economy , 66 (3), 233–255.
Tversky, A., & Kahneman, D. (1992). Advances in Prospect Theory: Cumulative
Representation of Uncertainty. Journal of Risk and Uncertainty , 5 (4), 297–323.
Uhlenbeck, G. E., & Ornstein, L. S. (1930). On the Theory of the Brownian Motion.
Physical Review , 36 (5), 823–841.
Vrugt, E. B., Bauer, R., Molenaar, R., & Steenkamp, T . (2007). Dynamic Commodity
T rading Strategies. In H. Till & J. Eagleeye (Eds.), Intelligent Commodity Investing:
New Strategies and Practical Insights for Informed Decision Making , Chapter 16.
London, UK: Risk Books.
Wang, C., & Yu, M. (2004). T rading Activity and Price Reversals in Futures Markets.
Journal of Banking & Finance , 28(6), 1337–1361.
Weiser, S. (2003, September 2003). The Strategic Case for Commodities in Portfolio
Diversiﬁcation. Commodities Now , pp. 7–11.
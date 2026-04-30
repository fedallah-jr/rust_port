# Chapter 8: Foreign Exchange (FX)

8
Foreign Exchange (FX)
8.1 Strategy: Moving Averages with HP Filter
In Sect. 3.12, we discussed a trading strategy for stocks wherein the trading
signal is based on 2 intersecting (shorter and longer) moving averages. A similar
approach can be applied to FX as well. However, FX spot rate time series tend
to be rather noisy, which can lead to false signals based on moving averages.
T o mitigate this, before computing the moving averages, the higher-frequency
noise can ﬁrst be ﬁltered out, e.g., using the so-called Hodrick–Prescott (HP)
ﬁlter.
1 Then, the remaining lower-frequency trend component (as opposed
to the raw spot rate) can be used to compute the moving averages and gen-
erate the trading signal (see, e.g., Harris and Yilmaz 2009). The HP ﬁlter is
given by:
S(t ) = S∗(t ) + ν(t )( 8.1)
g =
T∑
t =1
[
S(t ) − S∗(t )
]2 + λ
T −1∑
t =2
[
S∗(t + 1) − 2S∗(t ) + S∗(t − 1)
]2 (8.2)
g → min (8.3)
Here: the objective function g is minimized w.r.t. the set of T values of S∗(t ),
t = 1,..., T ; S(t ) i st h eF Xs p o tr a t ea tt i m e t ; S∗(t ) is the lower-frequency
1A.k.a. the Whittaker–Henderson method in actuarial sciences. For some pertinent literature, see, e.g.,
Baxter and King ( 1999), Bruder et al. ( 2013), Dao ( 2014), Ehlgen ( 1998), Harris and Yilmaz ( 2009),
Harvey and T rimbur (2008), Henderson (1924, 1925, 1938), Hodrick and Prescott ( 1997), Joseph (1952),
Lahmiri ( 2014), Mcelroy ( 2008), Weinert ( 2007), Whittaker ( 1923, 1924).
© The Author(s) 2018
Z. Kakushadze and J. A. Serur, 151 T rading Strategies,
https://doi.org/10.1007/978-3-030-02792-6_8
143

144 Z. Kakushadze and J. A. Serur
(“regular”) component; ν(t ) is the higher-frequency (“irregular”) component,
which is treated as noise; the ﬁrst term in Eq. ( 8.2) minimizes the noise,
while the second term (based on the discretized second derivative of S∗(t ))
penalizes the variation in S∗(t );a n d λ is the smoothing parameter. There is
no “fundamental” method to ﬁx λ. Sometimes (but not always) it is set to
λ = 100 ×n2,w h e r en is the data frequency measured on an annual basis (see,
e.g., Baxter and King ( 1999) for more detail). So, for monthly data, which is
what is usually used in this context, n = 12. The estimation period usually
spans several years (of monthly data). Once S∗(t ) is determined, two moving
averages MA(T1) and MA (T2), T1 < T2, are calculated based on S∗(t ).T h e n ,
as before, MA (T1)> MA(T2) is a buy signal, and MA (T1)< MA(T2) is a
sell signal.
8.2 Strategy: Carry T rade
Pursuant to “Uncovered Interest Rate Parity” (UIRP), excess interest earned in
one country compared with another country due to a differential between risk-
free interest rates in these countries would be precisely offset by depreciation
in the FX rate between their currencies:
(1 + rd ) = Et (S(t + T ))
S(t ) (1 + r f )( 8.4)
Here: rd is the domestic interest rate; r f is the foreign interest rate; both rd and
r f are assumed to be constant over the compounding period T ; S(t ) is the spot
FX rate at time t , which is the worth of 1 unit of the foreign currency in units
of the domestic currency; and Et (S(t + T )) is the future (at time t + T )s p o t
FX rate expected at time t .2 UIRP does not always hold, giving rise to trading
opportunities—which are not risk-free arbitrage opportunities (see below).
Thus, UIRP implies that high interest rate currencies should depreciate w.r.t.
low interest rate currencies, whereas empirically on average the opposite tends
to transpire, i.e., such currencies tend to appreciate (somewhat). 3 So, the basic
2Thus, 1 USD invested at time t in a risk-free asset in the U.S. would be forth (1 + rd ) USD at time
t + T . Alternatively, 1 USD would buy 1/S(t ) JPY at time t , which sum could be invested in a risk-free
asset in Japan at time t ,w h i c hw o u l db ew o r t h(1/S(t )) × (1 + r f ) JPY at time t + T , which in turn
could be expected to be exchanged for (Et (S(t + T ))/S(t )) × (1 + r f ) USD at time t + T . Requiring
that the U.S. and Japan investments yield the same return gives Eq. ( 8.4).
3This is known as “forward premium/discount anomaly/puzzle” or “Fama puzzle”. For some literature on
UIRP and related topics, see, e.g., Anker ( 1999), Ayuso and Restoy ( 1996), Baccheta and van Wincopp
(2006, 2010), Baillie and Osterberg ( 2000) ,B e k a e r te ta l .( 2007), Beyaert et al. ( 2007), Bilson ( 1981),
Chaboud and Wright ( 2005), Engel ( 1996), Fama ( 1984), Frachot ( 1996), Froot and Thaler ( 1990),

8 Foreign Exchange (FX) 145
carry strategy amounts to writing (i.e., selling) forwards on currencies that are
at a forward premium, i.e., the forward FX rate F (t, T ) exceeds the spot FX
rate S(t ), and buying forwards on currencies that are at a forward discount,
i.e., the forward FX rate F (t, T ) is lower than the spot FX rate S(t ).4 The
forward FX rate is given by 5
F (t, T ) = S(t ) 1 + rd
1 + r f
(8.5)
As mentioned above, the carry strategy 6 is not without risks: this trade can
generate losses if the borrowed (lent) currency suddenly appreciates (depreci-
ates) w.r.t. its counterpart, i.e., it is exposed to the FX rate risk. On the other
hand, if we borrow the low interest rate currency with the maturity date T ,
invest the funds in the high interest rate currency, and hedge this position
with a forward contract to exchange the high interest rate currency for the
low interest rate currency at maturity T (so we can cover the loan), ignoring
the transaction costs (and other subtleties such as taxes, etc.), this is a risk-free
position and any gains therefrom would amount to risk-free arbitrage. Hence
Eq. ( 8.5), which is a no-risk-free-arbitrage condition.
7
Strategy: High-minus-low Carry
The carry strategy discussed above can be applied to individual foreign curren-
cies. It can also be applied cross-sectionally, to multiple foreign currencies. Let
s(t ) = ln(S(t )) (log spot FX rate) and f (t, T ) = ln(F (t, T )) (log forward
Hansen and Hodrick ( 1980), Harvey ( 2015), Hodrick ( 1987), Ilut ( 2012), Lewis ( 1995), Lustig and
Ve rd e l h a n (2007), Mark and Wu ( 2001), Roll and Yan ( 2008).
4Ignoring transaction costs, this is equivalent to borrowing (lending) low (high) interest rate currencies
without hedging the FX rate risk.
5This is known as “Covered Interest Rate Parity” (CIRP). Note that, assuming Eq. ( 8.5) holds (see below),
when UIRP (i.e., Eq. 8.4) does not hold, F (t, T ) ̸= Et (S(t + T )).
6For some literature on currency carry trades and related topics, see, e.g., Bakshi and Panayotov ( 2013),
Brunnermeier et al. ( 2008), Burnside et al. ( 2007, 2008, 2011), Clarida et al. ( 2009), Deardorff ( 1979),
Doskov and Swinkels ( 2015), Hau ( 2014), Jurek ( 2014), Lustig et al. ( 2011, 2014), Olmo and Pilbeam
(2009), Ready et al. ( 2017), Rhee and Chang ( 1992).
7Nonetheless, deviations from CIRP (i.e., Eq. 8.5) do occur, which gives rise to covered interest arbitrage.
See, e.g., Akram et al. ( 2008), Avdjiev et al. ( 2016), Baba and Packer ( 2009), Boulos and Swanson ( 1994),
Clinton (1988), Coffey et al. ( 2009), Cosandier and Lang ( 1981), Du et al. ( 2018), Dufﬁe ( 2017), Frenkel
and Levich ( 1975), Frenkel and Levich ( 1981), Liao ( 2016), Mancini-Griffoli and Ranaldo ( 2011), Popper
(1993), Rime et al. ( 2017).

146 Z. Kakushadze and J. A. Serur
FX rate). The forward discount D(t, T ) is deﬁned as
D(t, T ) = s(t ) − f (t, T )( 8.6)
Pursuant to CIRP, Eq. ( 8.5), we have
D(t, T ) = ln
( 1 + r f
1 + rd
)
≈ r f − rd (8.7)
For a positive forward discount, we buy a forward (i.e., borrow the domes-
tic currency and invest in the foreign currency), and the higher the forward
discount, the more proﬁtable the strategy. For a negative forward discount,
we sell a forward (i.e., borrow the foreign currency and invest in the domestic
currency), and the lower the forward discount, the more proﬁtable the strategy.
So, we can construct a cross-sectional trade (including a zero-cost, i.e., dollar-
neutral trade—see, e.g., Lustig et al. 2011) by buying forwards on currencies
in some top quantile
8 by forward discount and selling forwards on currencies
in the corresponding bottom quantile. The forwards can, e.g., be one-month
forwards.
8.3 Strategy: Dollar Carry T rade
This strategy is based on the average cross-sectional forward discount D(t, T )
(see, e.g., Lustig et al. 2014)f o rab a s k e to f N foreign currencies:
D(t, T ) = 1
N
N∑
i =1
Di (t, T )( 8.8)
where Di (t, T ) is the forward discount for the currency labeled by i =
1,..., N . This strategy then goes long (short), with equal weights, all N
foreign currency forwards when D(t, T ) is positive (negative), where T can
be 1,2, 3,6,12 months. Empirical evidence suggests that this strategy relates
to the state of the U.S. economy, to wit, when the U.S. economy is weak, the
average forward discount tends to be positive. 9
8Unlike stocks, that number in thousands, there is a limited number of currencies to play with. Therefore,
one does not necessarily have the luxury of taking top and bottom deciles by forward discount. So, this
quantile can be a half, a third, etc., depending on the number of currencies.
9See, e.g., Cooper and Priestley ( 2008), Joslin and Konchitchki ( 2018), Joslin et al. ( 2014), Lustig et al.
(2014), Stambaugh ( 1988), Tille et al. ( 2001).

8 Foreign Exchange (FX) 147
8.4 Strategy: Momentum and Carry Combo
This is a combination of the momentum strategy (Sect. 8.1)10 and the carry
strategy (Sect. 8.2), or their variations. There is a variety of ways these strate-
gies can be combined (including an equally weighted combo, or some ideas
discussed in, e.g., Sects. 3.6 and 4.6). A simple combination is based on mini-
mizing the variance of the combo strategy using the sample covariance matrix of
historical returns R1(ts ) and R2(ts ) of the two strategies (see, e.g., Olszweski
and Zhou 2013). Let (here Var and Cor are serial variance and correlation,
respectively)
σ2
1 = Var(R1(ts )) ( 8.9)
σ2
2 = Var(R2(ts )) ( 8.10)
ρ = Cor(R1(ts ), R2(ts )) ( 8.11)
Then minimizing the historical variance of the combined return R(ts ) ﬁxes
the strategy weights w1 and w2:
R(ts ) = w1 R1(ts ) + w2 R2(ts )( 8.12)
w1 + w2 = 1 (8.13)
Var(R(ts )) → min (8.14)
w1 = σ2
2 − σ1σ2ρ
σ2
1 + σ2
2 − 2σ1σ2ρ (8.15)
w2 = σ2
1 − σ1σ2ρ
σ2
1 + σ2
2 − 2σ1σ2ρ (8.16)
8.5 Strategy: FX T riangular Arbitrage
This strategy is based on 3 currency pairs. 11 Let these currencies be A, B, and C.
Then we have 2 chains: (i) exchange A for B, exchange B for C, and exchange C
for A; and (ii) exchange A for C, exchange C for B, and exchange B for A. We
will focus on the ﬁrst chain as the second one is obtained by swapping B for C.
Each currency pair has the bid and the ask; e.g., Bid (A → B) and Ask (B →
10For additional literature on FX momentum strategies and related topics, see, e.g., Accominotti and
Chambers ( 2014), Ahmerkamp and Grant ( 2013), Burnside et al. ( 2011), Chiang and Jiang ( 1995),
Grobys et al. ( 2016), Menkhoff et al. ( 2012), Okunev and White ( 2003), Serban ( 2010).
11Albeit one can also consider more than 3 pairs, which is known as multi-currency arbitrage (see, e.g.,
Moosa 2003).

148 Z. Kakushadze and J. A. Serur
A)for the A-B pair. So, the rate at which A is exchanged into B is Bid (A → B),
while the rate at which B is exchanged into A is 1/Ask (B → A). Therefore,
Bid (B → A) = 1/Ask (B → A),a n d Ask (A → B) = 1/Bid (A → B).
In the chain (i) above, the trader starts with A and loops back to A with the
overall exchange rate
R(A → B → C → A) = Bid (A → B) × Bid (B → C) × 1
Ask (C → A) (8.17)
If this quantity is greater than 1, then the trader makes a proﬁt. Such oppor-
tunities are ephemeral, so fast market data and trade execution systems are
critical here. 12
References
Accominotti, O., & Chambers, D. (2014). Out-of-Sample Evidence on the
Returns to Currency T rading (Working Paper). Available online: https://ssrn.com/
abstract=2293684.
Ahmerkamp, J. D., & Grant, J. (2013). The Returns to Carry and Momentum Strategies
(Working Paper). Available online: https://ssrn.com/abstract=2227387.
Aiba, Y., & Hatano, N. (2006). A Microscopic Model of T riangular Arbitrage. Physica
A: Statistical Mechanics and Its Applications , 371(2), 572–584.
Aiba, Y., Hatano, N., T akayasu, H., Marumo, K., & Shimizu, T . (2002). T riangular
Arbitrage as an Interaction Among Foreign Exchange Rates. Physica A: Statistical
Mechanics and Its Applications , 310 (3–4), 467–479.
Aiba, Y., Hatano, N., T akayasu, H., Marumo, K., & Shimizu, T . (2003). T riangular
Arbitrage and Negative Auto-Correlation of Foreign Exchange Rates. Physica A:
Statistical Mechanics and Its Applications , 324 (1–2), 253–257.
Akram, Q. F ., Rime, D., & Sarno, L. (2008). Arbitrage in the Foreign Exchange
Market: T urning on the Microscope. Journal of International Economics , 76 (2),
237–253.
Anker, P . (1999). Uncovered Interest Parity, Monetary Policy and Time-Varying Risk
Premia. Journal of International Money and Finance , 18(6), 835–851.
Avdjiev, S., Du, W ., Koch, C., & Shin, H. S. (2016). The Dollar, Bank Leverage and the
Deviation from Covered Interest Parity (Working Paper). Available online: https://
ssrn.com/abstract=2870057.
Ayuso, J., & Restoy, F . (1996). Interest Rate Parity and Foreign Exchange Risk Premia
in the ERM. Journal of International Money and Finance , 15 (3), 369–382.
12For some literature on triangular arbitrage and related topics, see, e.g., Aiba and Hatano ( 2006), Aiba
et al. ( 2002, 2003), Akram et al. ( 2008), Choi ( 2011), Cross and Kozyakin ( 2015), Fenn et al. ( 2009),
Goldstein ( 1964), Gradojevic et al. ( 2017), Ito et al. ( 2012), Moosa ( 2001), Morisawa ( 2009), Mwangi
a n dD u n c a n(2012), Osu ( 2010).

8 Foreign Exchange (FX) 149
Baba, N., & Packer, F . (2009). Interpreting Deviations from Covered Interest Parity
During the Financial Market T urmoil of 2007–08. Journal of Banking & Finance ,
33(11), 1953–1962.
Bacchetta, P ., & van Wincoop, E. (2006). Incomplete Information Processing: A
Solution to the Forward Discount Puzzle. American Economic Review , 96 (3), 552–
576.
Bacchetta, P ., & van Wincoop, E. (2010). Infrequent Portfolio Decisions: A Solution
to the Forward Discount Puzzle. American Economic Review , 100 (3), 870–904.
Baillie, R. T ., & Osterberg, W . P . (2000). Deviations from Daily Uncovered Interest
Rate Parity and the Role of Intervention. Journal of International Financial Markets,
Institutions and Money , 10 (4), 363–379.
Bakshi, G., & Panayotov, G. (2013). Predictability of Currency Carry T rades and
Asset Pricing Implications. Journal of Financial Economics , 110 (1), 139–163.
Baxter, M., & King, R. (1999). Measuring Business Cycles: Approximate Band-Pass
Filters for Economic Time-Series. Review of Economics and Statistics , 81(4), 575–
593.
Bekaert, G., Wei, M., & Xing, Y. (2007). Uncovered Interest Rate Parity and the T erm
Structure. Journal of International Money and Finance , 26 (6), 1038–1069.
Beyaert, A., García-Solanes, J., & Pérez-Castejón, J. J. (2007). Uncovered Interest
Parity with Switching Regimes. Economic Modelling , 24 (2), 189–202.
Bilson, J. F . O. (1981). The “Speculative Efﬁciency” Hypothesis. Journal of Business ,
54 (3), 435–451.
Boulos, N., & Swanson, P . E. (1994). Interest Rate Parity in Times of T urbulence:
T h eI s s u eR e v i s i t e d .Journal of Financial and Strategic Decisions , 7 (2), 43–52.
Bruder, B., Dao, T .-L., Richard, R.-J., & Roncalli, T . (2013). T rend Filtering Meth-
ods for Momentum Strategies (Working Paper). Available online: https://ssrn.com/
abstract=2289097.
Brunnermeier, M. K., Nagel, S., & Pedersen, L. H. (2008). Carry T rades and Currency
Crashes. NBER Macroeconomics Annual , 23(1), 313–347.
Burnside, C., Eichenbaum, M., Kleshchelski, I., & Rebelo, S. (2011). Do Peso Prob-
lems Explain the Returns to the Carry T rade? Review of Financial Studies , 24 (3),
853–891.
Burnside, C., Eichenbaum, M., & Rebelo, S. (2007). The Returns to Currency Spec-
ulation in Emerging Markets. American Economic Review , 97 (2), 333–338.
Burnside, C., Eichenbaum, M., & Rebelo, S. (2008). Carry T rade: The Gains of
Diversiﬁcation. Journal of the European Economic Association , 6 (2/3), 581–588.
Burnside, C., Eichenbaum, M., & Rebelo, S. (2011). Carry T rade and Momentum
in Currency Markets. Annual Review of Financial Economics , 3, 511–535.
Chaboud, A. P ., & Wright, J. H. (2005). Uncovered Interest Parity: It Works, but Not
for Long. Journal of International Economics , 66 (2), 349–362.
Chiang, T . C., & Jiang, C. X. (1995). Foreign Exchange Returns over Short and Long
Horizons. International Review of Economics & Finance , 4 (3), 267–282.
Choi, M. S. (2011). Momentary Exchange Rate Locked in a T riangular Mechanism
of International Currency. Applied Economics , 43(16), 2079–2087.

150 Z. Kakushadze and J. A. Serur
Clarida, R. H., Davis, J. M., & Pedersen, N. (2009). Currency Carry T rade Regimes:
Beyond the Fama Regression. Journal of International Money and Finance , 28(8),
1375–1389.
Clinton, K. (1988). T ransactions Costs and Covered Interest Arbitrage: Theory and
Evidence. Journal of Political Economy , 96 (2), 358–370.
Coffey, N., Hrung, W . B., & Sarkar, A. (2009). Capital Constraints, Counterparty
Risk, and Deviations from Covered Interest Rate Parity (Federal Reserve Bank of
New York Staff Reports, No. 393). Available online: https://www.newyorkfed.org/
medialibrary/media/research/staff_reports/sr393.pdf .
Cooper, I., & Priestley, R. (2008). Time-Varying Risk Premiums and the Output Gap.
Review of Financial Studies , 22(7), 2801–2833.
Cosandier, P .-A., & Lang, B. R. (1981). Interest Rate Parity T ests: Switzerland and
Some Major Western Countries. Journal of Banking & Finance , 5 (2), 187–200.
Cross, R., & Kozyakin, V . (2015). Fact and Fictions in FX Arbitrage Processes. Journal
of Physics: Conference Series , 585, 012015.
Dao, T .-L. (2014). Momentum Strategies with the L1 Filter. Journal of Investment
Strategies, 3(4), 57–82.
Deardorff, A. V . (1979). One-Way Arbitrage and Its Implications for the Foreign
Exchange Markets. Journal of Political Economy , 87 (2), 351–364.
Doskov, N., & Swinkels, L. (2015). Empirical Evidence on the Currency Carry T rade,
1900–2012. Journal of International Money and Finance , 51, 370–389.
Du, W ., T epper, A., & V erdelhan, A. (2018). Deviations from Covered Interest
Rate Parity. Journal of Finance (forthcoming). https://doi.org/10.1111/joﬁ.12620 .
Available online: https://ssrn.com/abstract=2768207.
Dufﬁe, D. (2017, May). The Covered Interest Parity Conundrum. Risk. Available
online: https://www.risk.net/4353726.
Ehlgen, J. (1998). Distortionary Effects of the Optimal Hodrick-Prescott Filter. Eco-
nomics Letters , 61(3), 345–349.
Engel, C. (1996). The Forward Discount Anomaly and the Risk Premium: A Survey
of Recent Evidence. Journal of Empirical Finance , 3(2), 123–192.
Fama, E. F . (1984). Forward and Spot Exchange Rates. Journal of Monetary Economics ,
14 (3), 319–338.
Fenn, D. J., Howison, S. D., Mcdonald, M., Williams, S., & Johnson, N. F . (2009).
The Mirage of T riangular Arbitrage in the Spot Foreign Exchange Market. Inter-
national Journal of Theoretical and Applied Finance , 12(8), 1105–1123.
Frachot, A. (1996). A Reexamination of the Uncovered Interest Rate Parity Hypoth-
esis. Journal of International Money and Finance , 15 (3), 419–437.
Frenkel, J. A., & Levich, R. M. (1975). Covered Interest Arbitrage: Unexploited
Proﬁts? Journal of Political Economy , 83(2), 325–338.
Frenkel, J. A., & Levich, R. M. (1981). Covered Interest Arbitrage in the 1970’s.
Economics Letters , 8(3), 267–274.
Froot, K. A., & Thaler, R. H. (1990). Anomalies: Foreign Exchange. Journal of Eco-
nomic Perspectives , 4 (3), 179–192.

8 Foreign Exchange (FX) 151
Goldstein, H. N. (1964). The Implications of T riangular Arbitrage for Forward
Exchange Policy. Journal of Finance , 19 (3), 544–551.
Gradojevic, N., Gençay, R., & Erdemlioglu, D. (2017). Robust Prediction of T riangular
Currency Arbitrage with Liquidity and Realized Risk Measures: A New W avelet-Based
Ultra-High-Frequency Analysis (Working Paper). Available online: https://ssrn.com/
abstract=3018815.
Grobys, K., Heinonen, J.-P ., & Kolari, J. W . (2016). Is Currency Momentum a Hedge
for Global Economic Risk? (Working Paper). Available online: https://ssrn.com/
abstract=2619146.
Hansen, L. P ., & Hodrick, R. J. (1980). Forward Exchange Rates as Optimal Predictors
of Future Spot Rates: An Econometric Analysis. Journal of Political Economy , 88(5),
829–853.
Harris, R. D. F ., & Yilmaz, F . (2009). A Momentum T rading Strategy Based on the
Low Frequency Component of the Exchange Rate. Journal of Banking & Finance ,
33(9), 1575–1585.
Harvey, J. T . (2015). Deviations from Uncovered Interest Rate Parity: A Post Keynesian
Explanation. Journal of Post Keynesian Economics , 27 (1), 19–35.
Harvey, A., & T rimbur, T . (2008). T rend Estimation and the Hodrick-Prescott Filter.
Journal of the Japan Statistical Society , 38(1), 41–49.
Hau, H. (2014). The Exchange Rate Effect of Multi-currency Risk Arbitrage. Journal
of International Money and Finance , 47, 304–331.
Henderson, R. (1924). A New Method of Graduation. T ransactions of the Actuarial
Society of America , 25, 29–40.
Henderson, R. (1925). Further Remarks on Graduation. T ransactions of the Actuarial
Society of America , 26, 52–57.
Henderson, R. (1938). Mathematical Theory of Graduation . New York, NY: Actuarial
Society of America.
Hodrick, R. J. (1987). The Empirical Evidence on the Efﬁciency of Forward and Futures
Foreign Exchange Markets .N e wY o r k ,N Y :H a r w o o dA c a d e m i c .
Hodrick, R. J., & Prescott, E. C. (1997). Postwar U.S. Business Cycles: An Empirical
Investigation. Journal of Money, Credit and Banking , 29 (1), 1–16.
Ilut, C. (2012). Ambiguity Aversion: Implications for the Uncovered Interest Rate
Parity Puzzle. American Economic Journal: Macroeconomics , 4 (3), 33–65.
Ito, T ., Yamada, K., T akayasu, M., & T akayasu, H. (2012). Free Lunch! Arbitrage
Opportunities in the Foreign Exchange Markets (Working Paper). Available online:
http://www.nber.org/papers/w18541.
Joseph, A. (1952). The Whittaker-Henderson Method of Graduation. Journal of the
Institute of Actuaries , 78(1), 99–114.
Joslin, S., & Konchitchki, Y. (2018). Interest Rate V olatility, the Yield Curve, and the
Macroeconomy. Journal of Financial Economics , 128(2), 344–362.
Joslin, S., Priebsch, M., & Singleton, K. J. (2014). Risk Premiums in Dynamic T erm
Structure Models with Unspanned Macro Risks. Journal of Finance , 69 (3), 1197–
1233.

152 Z. Kakushadze and J. A. Serur
Jurek, J. W . (2014). Crash-Neutral Currency Carry T rades. Journal of Financial Eco-
nomics, 113(3), 325–347.
Lahmiri, S. (2014). Wavelet Low- and High-Frequency Components as Features for
Predicting Stock Prices with Backpropagation Neural Networks. Journal of King
Saud University—Computer and Information Sciences , 26 (2), 218–227.
Lewis, K. (1995). Puzzles in International Financial Markets. In G. M. Grossman
& K. Rogoff (Eds.), Handbook of International Economics ( V o l .3 ,C h a p t e r3 7 ) .
Amsterdam, The Netherlands: North-Holland.
Liao, G. Y. (2016). Credit Migration and Covered Interest Rate Parity (Working Paper).
Available online: http://scholar.harvard.edu/ﬁles/gliao/ﬁles/creditcip.pdf .
Lustig, H., Roussanov, N., & V erdelhan, A. (2011). Common Risk Factors in Currency
Markets. Review of Financial Studies , 24 (11), 3731–3777.
Lustig, H., Roussanov, N., & V erdelhan, A. (2014). Countercyclical Currency Risk
Premia. Journal of Financial Economics , 111(3), 527–553.
Lustig, H., & V erdelhan, A. (2007). The Cross-Section of Foreign Currency Risk
Premia and US Consumption Growth Risk. American Economic Review , 97 (1),
89–117.
Mancini-Griffoli, T ., & Ranaldo, A. (2011). Limits to Arbitrage During the Crisis:
Funding Liquidity Constraints and Covered Interest Parity (Working Paper). Available
online: https://ssrn.com/abstract=1549668.
Mark, N. C., & Wu, Y. (2001). Rethinking Deviations From Uncovered Interest Parity:
T h eR o l eo fC o v a r i a n c eR i s ka n dN o i s e .Economic Journal, 108(451), 1686–1706.
Mcelroy, T . (2008). Exact Formulas for the Hodrick-Prescott Filter. Econometrics Jour-
nal, 11(1), 208–217.
Menkhoff, L., Sarno, L., Schmeling, M., & Schrimpf, A. (2012). Currency Momen-
tum Strategies. Journal of Financial Economics , 106 (3), 660–684.
Moosa, I. A. (2001). T riangular Arbitrage in the Spot and Forward Foreign Exchange
Markets. Quantitative Finance , 1(4), 387–390.
Moosa, I. A. (2003). T wo-Currency, Three-Currency and Multi-Currency Arbitrage.
In International Financial Operations: Arbitrage, Hedging, Speculation, Financing and
Investment. Finance and Capital Markets Series (Chapter 1, pp. 1–18). London,
UK: Palgrave Macmillan.
Morisawa, Y. (2009). T oward a Geometric Formulation of T riangular Arbitrage: An
Introduction to Gauge Theory of Arbitrage. Progress of Theoretical Physics Supple-
ment, 179, 209–215.
Mwangi, C. I., & Duncan, M. O. (2012). An Investigation into the Existence of
Exchange Rate Arbitrage in the Mombasa Spot Market. International Journal of
Humanities and Social Science , 2(21), 182–196.
Okunev, J., & White, D. (2003). Do Momentum-Based Strategies Still Work in
Foreign Currency Markets? Journal of Financial and Quantitative Analysis , 38(2),
425–447.
Olmo, J., & Pilbeam, K. (2009). The Proﬁtability of Carry T rades. Annals of Finance ,
5 (2), 231–241.

8 Foreign Exchange (FX) 153
Olszweski, F ., & Zhou, G. (2013). Strategy Diversiﬁcation: Combining Momentum
and Carry Strategies Within a Foreign Exchange Portfolio. Journal of Derivatives &
Hedge Funds, 19 (4), 311–320.
Osu, B. O. (2010). Currency Cross Rate and T riangular Arbitrage in Nigerian
Exchange Market. International Journal of T rade, Economics and Finance, 1(4), 345–
348.
Popper, H. (1993). Long-T erm Covered Interest Parity: Evidence from Currency
Swaps. Journal of International Money and Finance , 12(4), 439–448.
Ready, R., Roussanov, N., & Ward, C. (2017). Commodity T rade and the Carry T rade:
A T ale of T wo Countries. Journal of Finance , 72(6), 2629–2684.
Rhee, S. G., & Chang, R. P . (1992). Intra-Day Arbitrage Opportunities in Foreign
Exchange and Eurocurrency Markets. Journal of Finance , 47 (1), 363–379.
Rime, D., Schrimpf, A., & Syrstad, O. (2017). Segmented Money Markets and Cov-
ered Interest Parity Arbitrage (Working Paper). Available online: https://ssrn.com/
abstract=2879904.
Roll, R., & Yan, S. (2008). An Explanation of the Forward Premium ‘Puzzle’. European
Financial Management , 6 (2), 121–148.
Serban, A. F . (2010). Combining Mean Reversion and Momentum T rading Strategies
in Foreign Exchange Markets. Journal of Banking & Finance , 34 (11), 2720–2727.
Stambaugh, R. F . (1988). The Information in Forward Rates: Implications for Models
of the T erm Structure. Journal of Financial Economics , 21(1), 41–70.
Tille, C., Stoffels, N., & Gorbachev, O. (2001). T o What Extent Does Productivity
Drive the Dollar? Current Issues in Economics and Finance , 7 (8), 1–6.
Weinert, H. L. (2007). Efﬁcient Computation for Whittaker-Henderson Smoothing.
Computational Statistics & Data Analysis , 52(2), 959–974.
Whittaker, E. T . (1923). On a New Method of Graduations. Proceedings of the Edin-
burgh Mathematical Society , 41, 63–75.
Whittaker, E. T . (1924). On the Theory of Graduation. Proceedings of the Royal Society
of Edinburgh , 44, 77–83.
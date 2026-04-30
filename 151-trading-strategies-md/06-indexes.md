# Chapter 6: Indexes

6
Indexes
6.1 Generalities
An index is a diversiﬁed portfolio of assets combined with some weights. The
underlying assets are often stocks, e.g., in indexes such as DJIA, S&P 500,
Russell 3000, etc. DJIA weights are based on price, while S&P 500 and Russell
3000 weights are based on market capitalization. Investment vehicles such as
index futures, index-based ETFs, etc., allow gaining exposure to a broad index
with a single trade.
1
6.2 Strategy: Cash-and-carry Arbitrage
This strategy (a.k.a. “index arbitrage”) aims to exploit price inefﬁciencies
between the index spot 2 price and index futures price. 3 Theoretically, the
price of the index futures must equal the spot price accounting for the cost of
carry during the life of the futures contract:
1For some literature on indexes, see, e.g., Antoniou and Holmes ( 1995), Beneish and Whaley ( 1996),
Bologna and Cavallo ( 2002), Bos ( 2000), Chang et al. ( 1999), Chiang and Wang ( 2002), Edwards ( 1988),
Frino et al. ( 2004), Graham and Pirie ( 1994), Hautcoeur ( 2006), Illueca and Lafuente ( 2003), Kenett et al.
(2013), Lamoureux and Wansley ( 1987), Larsen and Resnick ( 1998), Lo ( 2016), Schwartz and Laatsch
(1991), Spyrou ( 2005), Yo ( 2001).
2“Spot” refers to the current value of the index based on the current prices of its constituents. “Cash”
refers to the underlying index portfolio. This is common trader lingo.
3See, e.g., Brenner et al. ( 1989), Bühler and Kempf ( 1995), Butterworth and Holmes ( 2010), Chan and
Chung ( 1993), Cornell and French ( 1983), Dwyer et al. ( 1996), Fassas ( 2011), Puttonen ( 1993), Richie
et al. ( 2008), Yadav and Pope ( 1990), Yadav and Pope ( 1994).
© The Author(s) 2018
Z. Kakushadze and J. A. Serur, 151 T rading Strategies,
https://doi.org/10.1007/978-3-030-02792-6_6
121

122 Z. Kakushadze and J. A. Serur
F ∗(t, T ) = [S(t ) − D(t, T )] exp (r (T − t )) (6.1)
Here: F ∗(t, T ) is the theoretical (“fair”) price, at time t ,o ft h ef u t u r e sc o n t r a c t
with the delivery time T ; S(t ) is the spot value at time t ; D(t, T ) is the sum of
(discounted values of ) the dividends paid by the underlying stocks between the
time t and delivery; and r is the risk-free rate, which for the sake of simplicity
is assumed to be constant from t to delivery. 4 The basis is deﬁned as
B(t, T ) = F (t, T ) − F ∗(t, T )
S(t ) (6.2)
where F (t, T ) is the current price of the futures contract with the delivery
time T .I f B(t, T ) ̸= 0, more precisely, if |B(t, T )| exceeds the pertinent
transaction costs of executing the arbitrage trade, then there is an arbitrage
opportunity. If the basis is positive (negative), the futures price is rich (cheap)
compared with the spot price, so the arbitrage trade amounts to selling (buying)
the futures and buying (selling) the cash (i.e., the index basket).
5 The position
is closed when the basis goes to zero, i.e., the futures price converges to its fair
value. Such arbitrage opportunities are short-lived and with the advent of high
frequency trading require extremely fast execution. In many cases, the slippage
can be prohibitive to execute the trade. 6
6.3 Strategy: Dispersion T rading in Equity
Indexes
This strategy takes long positions on volatilities of the index constituents and
a short position on index volatility. It is rooted in an empirical observation
that, for the most part, 7 the implied volatility ˜σI from index options is sizably
higher than the theoretical index volatility σI given by
4Equation ( 6.1) further ignores some other pertinent aspects such as taxes, asymmetry of interest rates
(for long and short holdings), transaction costs, etc.
5Selling the futures poses no issues. However, selling the cash can be problematic with short-selling issues
such as hard-to-borrow securities, etc. Continuously maintaining a sizable dollar-neutral book which is
long cash and short futures can help circumvent such issues.
6In some cases incomplete baskets approximating the index can be executed to reduce the transaction
costs, e.g., in market cap weighted indexes, by omitting lower cap (and thus less liquid) stocks. However,
such mishedges also increase the risk of losing money on the trade.
7But not always—see below. For some literature on index vs. constituent volatilities and dispersion and
correlation trading, see, e.g., Carrasco ( 2007), Deng ( 2008), Lozovaia and Hizhniakova ( 2005), Marshall
(2008), Marshall ( 2009), Maze ( 2012), Meissner ( 2016), Nelken ( 2006).

6 Indexes 123
σ2
I =
N∑
i,j =1
wi wj σi σj ρij (6.3)
where wi are the weights of the stocks in the index, σi are their implied
volatilities from single-stock options, and ρij is the sample correlation matrix
(ρii = 1)8 computed based on a time series of historical returns. 9 Put differ-
ently, the index options are priced higher than the price corresponding to the
aforesaid theoretical volatility. So, a basic strategy can be structured as follows.
For each stock in the index we have a long position in n
i (near-ATM) single-
stock option straddles (whose payoffs are based on the stock prices Pi ), and
we have a short position in a (near-ATM) option straddle for the index (whose
payoff is based on the index level PI —see below), where
ni = Si PI
∑ N
i =1 Si Pi
(6.4)
Here: Si is shares outstanding for stock i (we are assuming the index is market
cap weighted); and PI is the index level. With this deﬁnition of ni ,w eh a v e
PI = ∑ N
i =1 ni Pi , so the index option straddle payoff matches the individual
single-stock option straddle payoffs as closely as possible. 10 All options have
approximately 1 month until the expiration, and all positions remain open
until the expiration. 11
Strategy: Dispersion T rading: Subset Portfolio
For some indexes, some component stocks may not have single-stock options.
Often these would be less liquid, lower market cap stocks. They would have
to be excluded from the bought portfolio. Reducing the number of bought
8Note that the pair-wise correlations ρij , i ̸= j , typically are unstable out-of-sample, which can introduce
a sizable error into this computation.
9For some pertinent literature, see, e.g., Bakshi and Kapadia ( 2003a, b), Bakshi et al. ( 2003), Bollen and
Whaley ( 2004), Branger and Schlag ( 2004), Coval and Shumway ( 2001), Dennis and Mayhew ( 2002),
Dennis et al. ( 2006), Driessen et al. ( 2009), Gârleanu et al. ( 2009), Lakonishok et al. ( 2007).
10If ATM options are not available for a given stock, OTM options (close to ATM) can be used.
11This strategy can be argued to be a volatility strategy. However, it can also be argued to be correlation
trading as the volatility of the portfolio depends on the correlations between its components (see Eq. ( 6.3)).
Thus, when the implied index volatility ˜σI is higher than the theoretical value σI , this can be (arguably)
interpreted as the implied average pair-wise correlation being higher than the average pair-wise correlation
based on ρ
ij . In this regard, at times the index implied volatility can be lower than its theoretical value,
so the dispersion strategy that is short index volatility would lose money and the reverse trade might be in
order. See, e.g., Deng ( 2008).

124 Z. Kakushadze and J. A. Serur
underlying single-stock options is also desirable to reduce transaction costs.
Furthermore, the sample correlation matrix ρij is singular for a typical lookback
period (e.g., daily close-to-close returns, going back 1 year, which is about 252
trading days) as the number of assets is large (500 for S&P 500 and even larger
for other indexes). As mentioned above, the pair-wise correlations are unstable
out-of-sample, which increases errors in the theoretical value σ
I computed via
Eq. ( 6.3). This can be mitigated as follows. 12
The singular and unstable correlation matrix can be made nonsingular and
more stable by replacing it with a statistical risk model (Kakushadze and Yu
2017). Let V (A)
i be the principal components of ρij with the eigenvalues λ(A)
in the decreasing order, λ(1) >λ (2) >λ (r ),w h e r e r is the rank of ρij (if
r < N , the other eigenvalues are null: λ(A) = 0, A > r ). The statistical risk
model correlation matrix is given by
ψij = ξ2
i δij +
K∑
A=1
λ(A) V (A)
i V (A)
j (6.5)
ξ2
i = 1 −
K∑
A=1
λ(A)
[
V (A)
i
]2
(6.6)
where K < r is the number of risk factors based on the ﬁrst K principal
components that are chosen to explain systematic risk, and ξi is the speciﬁc
(a.k.a. idiosyncratic) risk. The simplest way to ﬁx K is via eRank (effective
rank) (Roy and V etterli 2007)—see Kakushadze and Yu ( 2017)f o rd e t a i l sa n d
complete source code for constructing ψij and ﬁxing K .S o ,n o ww ec a nu s e
ψij (instead of ρij ) to compute the theoretical volatility σI :
σ2
I =
N∑
i,j =1
wi wj σi σj ψij =
N∑
i =1
w2
i σ2
i ξ2
i +
K∑
A=1
[ N∑
i =1
λ(A)V (A)
i wi σi
] 2
(6.7)
The ﬁrst term on the r.h.s. of Eq. ( 6.7) is due to the speciﬁc risk. The long
portfolio then contains only the straddles corresponding to the ﬁrst N∗ single-
stock options with the lowest N∗ values of w2
i σ2
i ξ2
i . E.g., for S&P 500 we can
take N∗ = 100.
12The variation of the dispersion trading strategy we discuss here is similar but not identical to the PCA
(principal component analysis) based strategy discussed in Deng ( 2008), Larsson and Flohr ( 2011), Su
(2006). The statistical risk model construction (see below) is more streamlined.

6 Indexes 125
6.4 Strategy: Intraday Arbitrage Between
Index ETFs
This strategy amounts to exploiting short-term mispricings between two ETFs
(call them ETF1 and ETF2) on the same underlying index. 13 It can be sum-
marized as follows:
Rule =
⎧
⎪⎪
⎪
⎨
⎪⎪
⎪
⎩
Buy ETF2, short ETF1 if P
Bid
1 ≥ P Ask
2 × κ
Liquidate position if P Bid
2 ≥ P Ask
1
Buy ETF1, short ETF2 if P Bid
2 ≥ P Ask
1 × κ
Liquidate position if P Bid
1 ≥ P Ask
2
(6.8)
Here: κ is a predeﬁned threshold, which is close to 1, e.g., κ = 1.002 (see, e.g.,
Marshall et al. 2013); P Bid
1 and P Bid
2 are the bid prices for ETF1 and ETF2,
and P Ask
1 and P Ask
2 are the ask prices. Marketable “ﬁll or kill” limit orders can
be used to execute the trades. Such arbitrage opportunities are ephemeral and
require a fast order execution system or else slippage will eat away the proﬁts.
6.5 Strategy: Index Volatility T argeting with
Risk-Free Asset
A volatility targeting strategy aims to maintain a constant volatility level, which
can be achieved by a periodic (weekly, monthly, etc.) rebalancing between a
risky asset—in this case an index—and a riskless asset (e.g., U.S. T reasury
bills).
14 If σ is the volatility of the risky asset 15 and the volatility target is σ∗,
then the allocation weight for the risky asset is given by 16 w = σ∗/σ,a n d
the allocation weight for the risk-free asset is 1 − w. T o avoid overtrading
13E.g., S&P 500 ETFs, SPDR T rust (ticker SPY) and iShares (ticker IVV). See, e.g., Marshall et al.
(2013). For some additional literature on ETF arbitrage and related topics, see, e.g., Abreu and Brun-
nermeier ( 2002), Ackert and Tian ( 2000), Ben-David et al. ( 2012), Brown et al. ( 2018), Cherry ( 2004),
Dolvin ( 2009), Garvey and Wu ( 2009), Hendershott and Moulton ( 2011), Johnson ( 2008), Maluf and
Albuquerque ( 2013).
14For some pertinent literature, see, e.g., Albeverio et al. ( 2013), Anderson et al. ( 2014), Cirelli et al.
(2017), Cooper ( 2010), Giese ( 2012), Khuzwayo and Maré ( 2014), Kim and Enke ( 2016), Kirby and
Ostdiek ( 2012), Papageorgiou et al. ( 2017), Perchet et al. ( 2014), T orricelli (2018), Zakamulin ( 2014).
15Usually, this is implied volatility as opposed to historical volatility as the former is considered to be
forward-looking. Alternatively, it can be based on various volatility-forecasting techniques.
16If there is a preset maximum leverage L,t h e n w is capped at L.

126 Z. Kakushadze and J. A. Serur
and reduce transaction costs, rebalancing (instead of periodically) can be done
based, e.g., on a preset threshold κ, say, only if the percentage change |/Delta1w|/w
since the last rebalancing exceeds κ.
References
Abreu, D., & Brunnermeier, M. K. (2002). Synchronization Risk and Delayed Arbi-
trage. Journal of Financial Economics , 66 (2–3), 341–360.
Ackert, L. F ., & Tian, Y. S. (2000). Arbitrage and Valuation in the Market for Standard
and Poor’s Depositary Receipts. Financial Management , 29 (3), 71–87.
Albeverio, S., Steblovskaya, V ., & Wallbaum, K. (2013). Investment Instruments with
V olatility T arget Mechanism.Quantitative Finance , 13(10), 1519–1528.
Anderson, R. M., Bianchi, S. W ., & Goldberg, L. R. (2014). Determinants of Levered
Portfolio Performance. Financial Analysts Journal , 70 (5), 53–72.
Antoniou, A., & Holmes, P . (1995). Futures T rading, Information and Spot Price
V olatility: Evidence from the FTSE 100 Stock Index Futures Contract Using
GARCH. Journal of Banking & Finance , 19 (1), 117–129.
Bakshi, G., & Kapadia, N. (2003a). Delta-Hedged Gains and the Negative Market
V olatility Risk Premium. Review of Financial Studies , 16 (2), 527–566.
Bakshi, G., & Kapadia, N. (2003b). V olatility Risk Premiums Embedded in Individual
Equity Options. Journal of Derivatives , 11(1), 45–54.
Bakshi, G., Kapadia, N., & Madan, D. (2003). Stock Return Characteristics, Skew
Laws, and the Differential Pricing of Individual Equity Options. Review of Financial
Studies, 16 (1), 101–143.
Ben-David, I., Franzoni, F . A., & Moussawi, R. (2012). ETFs, Arbitrage, and Contagion
(Working Paper). Available online: http://www.nccr-ﬁnrisk.uzh.ch/media/pdf/wp/
WP793_B1.pdf .
Beneish, M. D., & Whaley, R. E. (1996). An Anatomy of the “S&P Game”: The
Effects of Changing the Rules. Journal of Finance , 51(5), 1909–1930.
Bollen, N. P . B., & Whaley, R. (2004). Does Net Buying Pressure Affect the Shape of
Implied V olatility Functions? Journal of Finance , 59 (2), 711–754.
Bologna, P ., & Cavallo, L. (2002). Does the Introduction of Index Futures Effectively
Reduce Stock Market V olatility? Is the Futures Effect Immediate? Evidence from
the Italian Stock Exchange Using GARCH. Applied Financial Economics , 12(3),
183–192.
Bos, R. (2000). Index Calculation Primer . New York, NY: Standard and Poor’s Quan-
titative Services.
Branger, N., & Schlag, C. (2004). Why Is the Index Smile so Steep? Review of Finance ,
8(1), 109–127.
Brenner, M., Subrahmanyam, M. G., & Uno, J. (1989). Stock Index Futures Arbitrage
in the Japanese Markets. Japan and the World Economy , 1(3), 303–330.

6 Indexes 127
Brown, D. C., Davies, S., & Ringgenberg, M. (2018). ETF Arbitrage and Return Pre-
dictability (Working Paper). Available online: https://ssrn.com/abstract=2872414.
Bühler, W ., & Kempf, A. (1995). DAX Index Futures: Mispricing and Arbitrage in
German Markets. Journal of Futures Markets , 15 (7), 833–859.
Butterworth, D., & Holmes, P . (2010). Mispricing in Stock Index Futures Contracts:
Evidence for the FTSE 100 and FTSE mid 250 Contracts. Applied Economics Letters,
7 (12), 795–801.
Carrasco, C. G. (2007). Studying the Properties of the Correlation T rades (Work-
ing Paper). Available online: https://mpra.ub.uni-muenchen.de/22318/1/MPRA_
paper_22318.pdf .
Chan, K., & Chung, Y. P . (1993). Intraday Relationships Among Index Arbitrage,
Spot and Futures Price V olatility, and Spot Market V olume: A T ransactions Data
Te s t .Journal of Banking & Finance , 17 (4), 663–687.
Chang, E. C., Cheng, J. W ., & Pinegar, J. M. (1999). Does Futures T rading Increase
Stock Market V olatility? The Case of the Nikkei Stock Index Futures Exchange.
Journal of Banking & Finance , 23(5), 727–753.
Cherry, J. (2004). The Limits of Arbitrage: Evidence from Exchange T raded Funds (Work-
ing Paper). Available online: https://ssrn.com/abstract=628061.
Chiang, M. H., & Wang, C. Y. (2002). The Impact of Futures T rading on Spot Index
V olatility: Evidence from T aiwan Index Futures. Applied Economics Letters , 9 (6),
381–385.
Cirelli, S., Vitali, S., Ortobelli Lozza, S., & Moriggia, V . (2017). A Conservative
Discontinuous T arget V olatility Strategy. Investment Management and Financial
Innovations, 14 (2–1), 176–190.
Cooper, T . (2010). Alpha Generation and Risk Smoothing Using Managed Volatility
(Working Paper). Available online: https://ssrn.com/abstract=1664823.
Cornell, B., & French, K. R. (1983). The Pricing of Stock Index Futures. Journal of
Futures Markets, 3(1), 1–14.
Coval, J. D., & Shumway, T . (2001). Expected Options Returns. Journal of Finance ,
56 (3), 983–1009.
Deng, Q. (2008). Volatility Dispersion T rading (Working Paper). Available online:
https://ssrn.com/abstract=1156620.
Dennis, P ., & Mayhew, S. (2002). Risk-Neutral Skewness: Evidence from Stock
Options. Journal of Financial and Quantitative Analysis , 37 (3), 471–493.
Dennis, P ., Mayhew, S., & Stivers, C. (2006). Stock Returns, Implied V olatility Innova-
tions, and the Asymmetric V olatility Phenomenon. Journal of Financial and Quan-
titative Analysis , 41(2), 381–406.
Dolvin, S. D. (2009). ETFs: Arbitrage Opportunities and Market Forecasting. Journal
of Index Investing , 1(1), 107–116.
Driessen, J., Maenhout, P . J., & Vilkov, G. (2009). The Price of Correlation Risk:
Evidence from Equity Options. Journal of Finance ,
64 (3), 1377–1406.
Dwyer, G. P ., Jr., Locke, P ., & Yu, W . (1996). Index Arbitrage and Nonlinear Dynamics
Between the S&P 500 Futures and Cash. Review of Financial Studies , 9 (1), 301–
332.

128 Z. Kakushadze and J. A. Serur
Edwards, F . R. (1988). Futures T rading and Cash Market V olatility: Stock Index and
Interest Rate Futures. Journal of Futures Markets , 8(4), 421–439.
Fassas, A. P . (2011). Mispricing in Stock Index Futures Markets—The Case of Greece.
Investment Management and Financial Innovations , 8(2), 101–107.
Frino, A., Gallagher, D. R., Neubert, A. S., & Oetomo, T . N. (2004). Index Design and
Implications for Index T racking. Journal of Portfolio Management , 30 (2), 89–95.
Gârleanu, N., Pedersen, L. H., & Poteshman, A. M. (2009). Demand-Based Option
Pricing. Review of Financial Studies , 22(10), 4259–4299.
Garvey, R., & Wu, F . (2009). IntradayTime and Order Execution Quality Dimensions.
Journal of Financial Markets , 12(2), 203–228.
Giese, P . (2012). Optimal Design of V olatility-Driven Algo-alpha T rading Strategies.
Risk, 25 (5), 68–73.
Graham, S., & Pirie, W . (1994). Index Fund Rebalancing and Market Efﬁciency.
Journal of Economics and Finance , 18(2), 219–229.
Hautcoeur, P . C. (2006). Why and How to Measure Stock Market Fluctuations? The
Early History of Stock Market Indices, with Special Reference to the French Case (Work-
ing Paper). Available online: https://halshs.archives-ouvertes.fr/halshs-00590522/
PDF/wp200610.pdf .
Hendershott, T ., & Moulton, P . C. (2011). Automation, Speed, and Stock Market
Quality: The NYSE’s Hybrid. Journal of Financial Markets , 14 (4), 568–604.
Illueca, M., & Lafuente, J. A. (2003). The Effect of Spot and Futures T rading on Stock
Index V olatility: A Non-parametric Approach. Journal of Futures Markets , 23(9),
841–858.
Johnson, T . C. (2008). V olume, Liquidity, and Liquidity Risk. Journal of Financial
Economics, 87 (2), 388–417.
Kakushadze, Z., & Yu, W . (2017). Statistical Risk Models. Journal of Investment Strate-
gies, 6 (2): 1–40. Available online: https://ssrn.com/abstract=2732453.
Kenett, D. Y., Ben-Jacob, E., Stanley, H. E., & gur-Gershgoren, G. (2013). How High
Frequency T rading Affects a Market Index. Scientiﬁc Reports , 3, 2110.
Khuzwayo, B., & Maré, E. (2014). Aspects of V olatility T argeting for South African
Equity Investors. South African Journal of Economic and Management Sciences ,
17 (5), 691–699.
Kim, Y., & Enke, D. (2016). Using Neural Networks to Forecast V olatility for an
Asset Allocation Strategy Based on the T arget V olatility. Procedia Computer Science ,
95, 281–286.
Kirby, C., & Ostdiek, B. (2012). It’s All in the Timing: Simple Active Portfolio Strate-
gies That Outperform Naïve Diversiﬁcation. Journal of Financial and Quantitative
Analysis, 47 (2), 437–467.
Lakonishok, J., Lee, I., Pearson, N. D., & Poteshman, A. M. (2007). Option Market
Activity. Review of Financial Studies , 20 (3), 813–857.
Lamoureux, C., & Wansley, J. (1987). Market Effects of Changes in the S&P 500
Index. Financial Review , 22(1), 53–69.
Larsen, G., & Resnick, B. (1998). Empirical Insights on Indexing. Journal of Portfolio
Management, 25 (1), 51–60.

6 Indexes 129
Larsson, P ., & Flohr, L. (2011). Optimal Proxy-Hedging of Options on Illiquid Baskets
(Working Paper). Available online: https://www.math.kth.se/matstat/seminarier/
reports/M-exjobb11/110131a.pdf .
Lo, A. (2016). What Is an Index? Journal of Portfolio Management , 42(2), 21–36.
Lozovaia, T ., & Hizhniakova, H. (2005). How to Extend Modern Portfolio Theory to
Make Money from T rading Equity Options (Working Paper). Available online: http://
www.ivolatility.com/doc/Dispersion_Article.pdf.
Maluf, Y. S., & Albuquerque, P . H. M. (2013). Empirical Evidence: Arbitrage with
Exchange-T raded Funds (ETFs) on the Brazilian Market. Revista Contabilidade &
Finanças , 24 (61), 64–74.
Marshall, C. M. (2008). Volatility T rading: Hedge Funds and the Search for Alpha
(New Challenges to the Efﬁcient Markets Hypothesis) . Ph.D. thesis, Fordham Univer-
sity, New York, NY. Available online: https://fordham.bepress.com/dissertations/
AAI3353774/.
Marshall, C. M. (2009). Dispersion T rading: Empirical Evidence from U.S. Options
Markets. Global Finance Journal , 20 (3), 289–301.
Marshall, B. R., Nguyen, N. H., & Visaltanachoti, N. (2013). ETF Arbitrage: Intraday
Evidence. Journal of Banking & Finance , 37 (9), 3486–3498.
Maze, S. (2012). Dispersion T rading in South Africa: An Analysis of Proﬁtability
and a Strategy Comparison (Working Paper). Available online: https://ssrn.com/
abstract=2398223.
Meissner, G. (2016). Correlation T rading Strategies: Opportunities and Limitations.
Journal of T rading, 11(4), 14–32.
Nelken, I. (2006). Variance Swap V olatility Dispersion. Derivatives Use, T rading &
Regulation, 11(4), 334–344.
Papageorgiou, N. A., Reeves, J. J., & Sherris, M. (2017). Equity Investing with T argeted
Constant Volatility Exposure (Working Paper). Available online: https://ssrn.com/
abstract=2614828.
Perchet, R., de Carvalho, R. L., & Moulin, P . (2014). Intertemporal Risk Parity: A
Constant V olatility Framework for Factor Investing. Journal of Investment Strategies ,
4 (1), 19–41.
Puttonen, V . (1993). The Ex ante Proﬁtability of Index Arbitrage in the New Finnish
Markets. Scandinavian Journal of Management , 9 (S1), 117–127.
Richie, N., Daigler, R. T ., & Gleason, K. C. (2008). The Limits to Stock Index
Arbitrage: Examining S&P 500 Futures and SPDRS. Journal of Futures Markets ,
28(12), 1182–1205.
Roy, O., & V etterli, M. (2007, September 3–7). The Effective Rank: A Measure of
Effective Dimensionality. In: Proceedings—EUSIPCO 2007, 15th European Signal
Processing Conference (pp. 606–610). Pozna ´n, Poland.
Schwartz, T . V ., & Laatsch, F . (1991). Price Discovery and Risk T ransfer in Stock
Index Cash and Futures Markets. Journal of Futures Markets , 11(6), 669–683.
Spyrou, S. I. (2005). Index Futures T rading and Spot Price V olatility: Evidence from
an Emerging Market. Journal of Emerging Market Finance , 4 (2), 151–167.

130 Z. Kakushadze and J. A. Serur
Su, X. (2006). Hedging Basket Options by Using a Subset of Underlying Assets (Work-
ing Paper). Available online: https://www.econstor.eu/bitstream/10419/22959/1/
bgse14_2006.pdf .
T orricelli, L. (2018). Volatility T argeting Using Delayed Diffusions (Working Paper).
Available online: https://ssrn.com/abstract=2902063.
Yadav, P . K., & Pope, P . F . (1990). Stock Index Futures Arbitrage: International Evi-
dence. Journal of Futures Markets , 10 (6), 573–603.
Yadav, P . K., & Pope, P . F . (1994). Stock Index Futures Mispricing: Proﬁt Opportunities
or Risk Premia? Journal of Banking & Finance , 18(5), 921–953.
Yo, S. W . (2001). Index Futures T rading and Spot Price V olatility. Applied Economics
Letters, 8(3), 183–186.
Zakamulin, V . (2014). Dynamic Asset Allocation Strategies Based on Unexpected
V olatility.Journal of Alternative Investments , 16 (4), 37–50.
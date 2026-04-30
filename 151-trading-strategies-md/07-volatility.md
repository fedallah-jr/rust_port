# Chapter 7: Volatility

7
Volatility
7.1 Generalities
Some option trading strategies discussed in Chapter 2 are volatility strategies,
in the sense that they make bets on high or low future volatility. 1 There are
various ways to make volatility bets, and volatility can be viewed as an asset class
of its own. Historical volatility is based on a time series of past returns. In con-
trast, implied volatility extracted from options is considered a forward-looking
measure of volatility.
2 VIX (CBOE Volatility Index, a.k.a. the “uncertainty
index” or the “fear gauge index”) 3 and other volatility indexes 4 and derivatives
(options and futures) on volatility indexes such as VIX provide avenues for
volatility trading.
7.2 Strategy: VIX Futures Basis Trading
This is essentially a mean-reversion strategy. It is rooted in empirical obser-
vations (see, e.g., Mixon 2007; Nossman and Wilhelmsson 2009;S i m o na n d
1E.g., long (short) straddles bet on increasing (decreasing) volatility.
2See, e.g., Abken and Nandi ( 1996), Ané and Labidi ( 2001), Canina and Figlewski ( 1993), Christensen
and Prabhala ( 1998), Derman and Kani ( 1994) ,D u m a se ta l .(1998), Dupire ( 1994), Glasserman and Wu
(2010), He et al. ( 2015), Lamoureux and Lastrapes ( 1993), Mayhew ( 1995), Skiadopoulos et al. ( 1999).
3See, e.g., Äijö ( 2008), Corrado and Miller ( 2005), Fleming et al. ( 1995), Maghrebi et al. ( 2007), Shaikh
and Padhi ( 2015), Siriopoulos and Fassas ( 2009), Skiadopoulos ( 2004), Whaley ( 2000), Whaley ( 2009).
4E.g., RVX (CBOE Russell 2000 Volatility Index), VXEEM (CBOE Emerging Markets ETF Volatility
Index), TYVIX (CBOE/CBOT 10-year U.S. T reasury Note Volatility Index), GVZ (CBOE Gold ETF
Volatility Index), EUVIX (CBOE/CME FX Euro Volatility Index), VXGOG (CBOE Equity VIX on
Google), VVIX (CBOE VIX of VIX Index), etc.
© The Author(s) 2018
Z. Kakushadze and J. A. Serur, 151 T rading Strategies,
https://doi.org/10.1007/978-3-030-02792-6_7
131

132 Z. Kakushadze and J. A. Serur
Campasano 2014)5 that the VIX futures basis (deﬁned below) has essentially
no forecasting power for subsequent VIX changes but has substantial forecast-
ing power for subsequent VIX futures price changes. The VIX futures basis
BVIX (for our purposes here) is deﬁned as
BVIX = PUX 1 − PVIX (7.1)
D = BVIX
T (7.2)
Here: PUX 1 is the price of the ﬁrst-month contract VIX futures 6; PVIX is
the VIX price; D is the daily roll value; and T is the number of business
days until the settlement (which is assumed to be at least 10). Empirically, the
futures prices tend to fall for positive basis and rise for negative basis (mean-
reversion). So, the strategy amounts to shorting VIX futures when the VIX
futures curve is upward-sloping (a.k.a. “contango”, so the basis is positive),
and buying VIX futures when the VIX futures curve is downward-sloping
(a.k.a.“backwardation”, so the basis is negative). Here is a simple trading rule
(see, e.g., Simon and Campasano 2014):
Rule =
⎧
⎪⎪
⎪⎨
⎪⎪
⎪
⎩
Open long UX1 position if D < −0.10
Close long UX1 position if D > −0.05
Open short UX1 position if D > 0.10
Close short UX1 position if D < 0.05
(7.3)
A short (long) UX1 position is exposed to a risk of a sudden increase (decrease)
in the volatility, which typically occurs during equity market sell-offs (rallies),
so this risk can be hedged by, e.g., shorting (buying) mini-S&P 500 futures.
7
The hedge ratio can be estimated, e.g., based on a historical serial regression
of the VIX futures price changes over the front-month mini-S&P 500 futures
contract returns. 8
5For some additional literature on VIX futures basis and related topics, see, e.g., Buetow and Henderson
(2016), Donninger ( 2014), Fu et al. ( 2016), Lee et al. ( 2017), Zhang et al. ( 2010), Zhang and Zhu ( 2006).
6UX1 has approximately 1 month to maturity, UX2 has approximately 2 months, etc.
7T ypically, VIX and the equity markets are anti-correlated.
8See, e.g., Simon and Campasano ( 2014) for details.

7 Volatility 133
7.3 Strategy: Volatility Carry with T wo ETNs
VXX is an exchange-traded note (ETN) that tracks VIX via a portfolio of
short-maturity (months 1 and 2) VIX futures contracts. T o maintain a constant
maturity, at the close of each day, a portion of the shorter-maturity futures is
sold and replaced with the longer-maturity futures bought with the proceeds.
Since the VIX futures curve is in contango most of the time, the longer-maturity
futures are priced higher than the shorter-maturity futures, so this rebalancing
amounts to a decay in the value of VXX over time, which is known as the roll
(or contango) loss. Further, as time passes, the futures converge to the spot
(VIX), so VXX loses value so long as the VIX futures curve is in contango.
VXZ is yet another ETN that tracks VIX via a portfolio of medium-maturity
(months 4 through 7) VIX futures. VXZ also suffers roll loss, but to a lesser
degree than VXX as the slope of the VIX futures curve, when in contango,
decreases with maturity.
9 The basic strategy then is to short VXX and buy
VXZ with the hedge ratio that can be determined via a serial regression. 10
This strategy is not without risks, however. There can be short-term spikes in
VXX (the corresponding spikes in VXZ usually are sizably smaller), which can
lead to substantial short-term P&L drawdowns, even if the strategy is overall
proﬁtable.
Strategy: Hedging Short VXX with VIX Futures
Instead of using a long position in VXZ to hedge the short position in VXX,
one can directly use a basket of, e.g., medium-maturity VIX futures.
11 The
N VIX futures have some weights wi . These weights can be ﬁxed in a variety of
ways, e.g., by minimizing the tracking error, i.e., by running a serial regression
(with the intercept) of VXX returns over the N futures returns. Then we have:
wi = σX
N∑
j =1
C−1
ij σj ρj (7.4)
9For some literature on volatility ETNs and related topics, see, e.g., Alexander and Korovilas ( 2012),
Avellaneda and Papanicolaou ( 2018), DeLisle et al. ( 2014), Deng et al. ( 2012), Eraker and Wu ( 2014),
Gehricke and Zhang ( 2018), Grasselli and Wagalath ( 2018), Hancock ( 2013), Husson and McCann
(2011), Liu and Dash ( 2012), Liu et al. ( 2018), Moran and Dash ( 2007).
10We have h = β = ρσX /σZ ,w h e r e :h (known as the optimal hedge ratio) is the number of VXZ to
buy for each VXX shorted; β is the coefﬁcient (for the VXZ returns) of the serial regression (with the
intercept) of the VXX returns over the VXZ returns; σX and σZ are the historical volatilities of VXX and
VXZ, respectively; and ρ is the pair-wise historical correlation between VXX and VXZ.
11These can have maturities of, e.g., 4 through 7 months (thus mimicking the VXZ composition).

134 Z. Kakushadze and J. A. Serur
Here: ρi is the pair-wise historical correlation between the futures labeled by
i and VXX; Cij is the N × N sample covariance matrix for the N futures
(σ2
i = Cii is the historical variance for the futures labeled by i ); and σX is
the historical volatility of VXX. Some wi may turn out to be negative. This
is not necessarily an issue, but one may wish to impose the bounds wi ≥ 0.
Further, one may wish the strategy to be dollar-neutral, which would amount
to imposing the constraint
N∑
i =1
wi = 1 (7.5)
which the optimal hedge ratios ( 7.4) generally do not satisfy. Also, instead of
minimizing the tracking error, one may wish to minimize the variance of the
entire portfolio. And so on. The portfolio can be rebalanced monthly or more
frequently.
7.4 Strategy: Volatility Risk Premium
Empirical evidence indicates that implied volatility tends to be higher than
realized volatility most of the time, which is known as the “volatility risk
premium”.12 Simply put, most of the time options are priced higher than
the prices one would expect based on realized volatility, so the idea is to sell
volatility. E.g., the trader can sell straddles based on S&P 500 options. As a
possible proxy for volatility risk premium, the trader can, e.g., use the difference
between VIX at the beginning of the current month and the realized volatility
(in %, as VIX is quoted in %) of S&P 500 daily returns since the beginning
of the current month. If the spread is positive, the trader sells the straddle. If
the volatility spikes (which usually happens if the market sells of ), the strategy
will lose money. It is proﬁtable in sideways markets.
13
12For some pertinent literature, see, e.g., Bakshi and Kapadia ( 2003a), Bollerslev et al. ( 2011), Carr and
Wu ( 2009, 2016), Christensen and Prabhala ( 1998), Eraker ( 2009), Ge ( 2016), Miao et al. ( 2012),
Prokopczuk and Simen ( 2014), Saretto and Goyal ( 2009), T odorov (2010).
13Also, index options are better suited for this strategy than single-stock options as index options typically
have higher volatility risk premia (see Sect. 6.3).

7 Volatility 135
Strategy: Volatility Risk Premium with Gamma Hedging
The ATM straddles in the above strategy are Delta-neutral. 14 S o ,t h i si sa
“Vega play”, i.e., the trader is shorting Vega. If the underlying (S&P 500)
moves, the short straddle is no longer Delta-neutral: if the underlying goes up
(down), Delta becomes negative (positive). So a variation of this strategy is
to use Gamma hedging to keep the strategy close to Delta-neutral, which is
achieved by buying (selling) the underlying if it moves up (down). Then this
becomes a “Theta play”, i.e., the strategy now aims to capitalize on the Theta-
decay of the value of the sold options. So, the price of this is the cost of the
Gamma hedging, which reduces the P&L. As the underlying moves more and
more away from the strike of the sold put and call options, the Gamma hedge
becomes more and more expensive and eventually will exceed the collected
option premia, at which point the strategy starts losing money.
7.5 Strategy: Volatility Skew—Long Risk
Reversal
OTM put options with the underlying at S0 = K +κ tend to be priced higher
than OTM call options with the underlying at S0 = K − κ (here K is the
strike price, and κ> 0 is the distance from the strike). I.e., with all else being
equal, the implied volatility for puts is higher than for calls. 15 The long risk
reversal strategy (see Sect. 2.12), where the trader buys an OTM call option
and sells an OTM put option, captures this skew. However, this is a directional
strategy—it loses money if the price of the underlying drops below K put −C,
where K put is the strike price of the put, and C > 0 is the premium of the
put minus the premium of the call.
14Some of the Greeks for options are: /Theta1= ∂V /∂t (Theta), /Delta1= ∂V /∂S (Delta), /Gamma1= ∂2 V /∂S2
(Gamma), ν = ∂V /∂σ (Vega). Here: V is the value of the option; t is time; S is the price of the
underlying; σ is the implied volatility.
15For some pertinent literature, see, e.g., Bondarenko ( 2014), Chambers et al. ( 2014), Corrado and Su
(1997), Damghani and Kos ( 2013), DeMiguel et al. ( 2013), Doran and Krieger ( 2010), Doran et al.
(2007), Fengler et al. ( 2012), Flint and Maré ( 2017), Jackwerth ( 2000), Kozhan et al. ( 2013), Liu et al.
(2016), Mixon ( 2011), Zhang and Xiang ( 2008).

136 Z. Kakushadze and J. A. Serur
7.6 Strategy: Volatility Trading with Variance
Swaps
One issue with trading volatility using options is the need to (almost con-
tinuously) Delta-hedge the position to avoid directional exposure, 16 which
practically can be both cumbersome and costly. T o avoid the need for Delta-
hedging, one can make volatility bets using variance swaps. A variance swap is
a derivative contract whose payoff P(T ) at maturity T is proportional to the
difference between the realized variance v(T ) of the underlying and the preset
variance strike K :
P(T ) = N × (v(T ) − K ) (7.6)
v(T ) = F
T
T∑
t =1
R2(t )( 7.7)
R(t ) = ln
[ S(t )
S(t − 1)
]
(7.8)
Here: t = 0,1,..., T labels sample points (e.g., trading days); S(t ) is the
price of the underlying at time t ; R(t ) is the log-return from t − 1 to t ; F
is the annualization factor (thus, if t labels trading days, then F = 252);
and N is the “variance notional”, which is preset. Note that in Eq. ( 7.7)t h e
mean of R(t ) over the period t = 1 to t = T is not subtracted, hence T in
the denominator.17 Long (short) variance swap is a bet that the future realized
volatility will be higher (lower) than the current implied volatility. Long (short)
variance swaps can therefore be used instead of, e.g., long (short) straddles to
go long (short) volatility. For instance, the dispersion strategy of Sect. 6.3 can
be executed by selling a variance swap on an index and buying variance swaps
on the index constituents (cf. selling and buying straddles).
18
16See Sect. “Strategy: Volatility Risk Premium with Gamma Hedging ” for a Delta-hedging strategy (a.k.a.
“Gamma scalping”).
17If the mean is subtracted, then the denominator would be T − 1 instead.
18For some literature on variance swaps, see, e.g., Aït-Sahalia et al. ( 2015), Bernard et al. ( 2014), Broadie
and Jain ( 2008), Bossu ( 2006), Carr and Lee ( 2007), Carr and Lee ( 2009), Carr et al. ( 2012), Demeterﬁ
et al. ( 1999), Elliott et al. ( 2007), Filipovi´ce ta l .( 2016), Hafner and Wallmeier ( 2007), Härdle and
Silyakova (2010), Jarrow et al. ( 2013), Konstantinidi and Skiadopoulos ( 2016), Leontsinis and Alexander
(2016), Liverance ( 2010), Martin ( 2011), Rujivan and Zhu ( 2012), Schoutens ( 2005), Wystup and Zhou
(2014), Zhang ( 2014), Zheng and Kwok ( 2014).

7 Volatility 137
References
Abken, P . A., & Nandi, S. (1996). Options and Volatility. Federal Reserve Bank of
Atlanta, Economic Review , 81(3), 21–35.
Äijö, J. (2008). Implied Volatility T erm Structure Linkages Between VDAX, VSMI
and VSTOXX Volatility Indices. Global Finance Journal , 18(3), 290–302.
Aït-Sahalia, Y., Karaman, M., & Mancini, L. (2015). The T erm Structure of Vari-
ance Swaps and Risk Premia (Working Paper). Available online: https://ssrn.com/
abstract=2136820.
Alexander, C., & Korovilas, D. (2012). Understanding ETNs on VIX Futures (Working
Paper). Available online: https://ssrn.com/abstract=2043061.
Ané, T ., & Labidi, C. (2001). Implied Volatility Surfaces and Market Activity Over
Time. Journal of Economics and Finance , 25 (3), 259–275.
Avellaneda, M., & Papanicolaou, A. (2018). Statistics of VIX Futures and Applications
to T rading Volatility Exchange-T raded Products. Journal of Investment Strategies ,
7 (2), 1–33.
Bakshi, G., & Kapadia, N. (2003). Delta-Hedged Gains and the Negative Market
Volatility Risk Premium. Review of Financial Studies , 16 (2), 527–566.
Bernard, C., Cui, Z., & Mcleish, D. (2014). Convergence of the Discrete Variance
Swap in Time-Homogeneous Diffusion Models. Quantitative Finance Letters , 2(1),
1–6.
Bollerslev, T ., Gibson, M., & Zhou, H. (2011). Dynamic Estimation of Volatility Risk
Premia and Investor Risk Aversion from Option-Implied and Realized Volatilities.
Journal of Econometrics , 160 (1), 235–245.
Bondarenko, O. (2014). Why Are Put Options so Expensive? Quarterly Journal of
Finance, 4 (3), 1450015.
Bossu, S. (2006, March). Introduction to Variance Swaps. Wilmott Magazine ,p p .
50–55.
Broadie, M., & Jain, A. (2008). The Effect of Jumps and Discrete Sampling on Volatil-
ity and Variance Swaps. International Journal of Theoretical and Applied Finance ,
11(8), 761–797.
Buetow, G. W ., & Henderson, B. J. (2016). The VIX Futures Basis: Determinants
and Implications. Journal of Portfolio Management , 42(2), 119–130.
Canina, L., & Figlewski, S. (1993). The Informational Content of Implied Volatility.
Review of Financial Studies , 6 (3), 659–681.
Carr, P ., & Lee, R. (2007). Realized Volatility and Variance: Options via Swaps. Risk,
20 (5), 76–83.
Carr, P ., & Lee, R. (2009). Volatility Derivatives.Annual Review of Financial Economics ,
1, 319–339.
Carr, P ., & Wu, L. (2009). Variance Risk Premiums.Review of Financial Studies , 22(3),
1311–1341.
Carr, P ., & Wu, L. (2016). Analyzing Volatility Risk and Risk Premium in Option
Contracts: A New Theory. Journal of Financial Economics
, 120 (1), 1–20.

138 Z. Kakushadze and J. A. Serur
Carr, P ., Lee, R., & Wu, L. (2012). Variance Swaps on Time-Changed Lévy Processes.
Finance and Stochastics , 16 (2), 335–355.
Chambers, D. R., Foy, M., Liebner, J., & Lu, Q. (2014). Index Option Returns: Still
Puzzling. Review of Financial Studies , 27 (6), 1915–1928.
Christensen, B. J., & Prabhala, N. R. (1998). The Relation Between Implied and
Realized Volatility. Journal of Financial Economics , 50 (2), 125–150.
Corrado, C. J., & Miller, T . W , Jr. (2005). The Forecast Quality of CBOE Implied
Volatility Indexes. Journal of Futures Markets , 25 (4), 339–373.
Corrado, C. J., & Su, T . (1997). Implied Volatility Skews and Stock Return Skewness
and Kurtosis Implied by Stock Option Prices. European Journal of Finance , 3(1),
73–85.
Damghani, B. M., & Kos, A. (2013). De-arbitraging With a Weak Smile: Application
to Skew Risk. Wilmott Magazine , 2013(64), 40–49.
DeLisle, J., Doran, J., & Krieger, K. (2014). Volatility as an Asset Class: Holding VIX in
a Portfolio (Working Paper). Available online: https://ssrn.com/abstract=2534081.
Demeterﬁ, K., Derman, E., Kamal, M., & Zou, J. (1999). A Guide to Volatility and
Variance Swaps. Journal of Derivatives , 6 (4), 9–32.
DeMiguel, V ., Plyakha, Y., Uppal, R., & Vilkov, G. (2013). Improving Portfolio
Selection Using Option-Implied Volatility and Skewness. Journal of Financial and
Quantitative Analysis , 48(6), 1813–1845.
Deng, G., McCann, C., & Wang, O. (2012). Are VIX Futures ETPs Effective Hedges?
Journal of Index Investing , 3(3), 35–48.
Derman, E., & Kani, I. (1994). Riding on a Smile. Risk, 7 (2), 139–145.
Donninger, C. (2014). VIX Futures Basis T rading: The Calvados-Strategy 2.0 (Working
Paper). Available online: https://ssrn.com/abstract=2379985.
Doran, J. S., & Krieger, K. (2010). Implications for Asset Returns in the Implied
Volatility Skew. Financial Analysts Journal , 66 (1), 65–76.
Doran, J. S., Peterson, D. R., & T arrant, B. C. (2007). Is There Information in the
Volatility Skew? Journal of Futures Markets , 27 (10), 921–959.
Dumas, B., Fleming, J., & Whaley, R. (1998). Implied Volatility Functions: Empirical
T ests.Journal of Finance , 53(6), 2059–2106.
Dupire, B. (1994). Pricing with a Smile. Risk, 7 (1), 18–20.
Elliott, R., Siu, T ., & Chan, L. (2007). Pricing Volatility Swaps Under Heston’s
Stochastic Volatility Model with Regime Switching. Applied Mathematical Finance ,
14 (1), 41–62.
Eraker, B. (2009). The Volatility Premium (Working Paper). Available online: http://
www.nccr-ﬁnrisk.uzh.ch/media/pdf/Eraker_23-10.pdf .
Eraker, B., & Wu, Y. (2014). Explaining the Negative Returns to VIX Futures and
ETNs: An Equilibrium Approach (Working Paper). Available online: https://ssrn.
com/abstract=2340070.
Fengler, M. R., Herwartz, H., & Werner, C. (2012). A Dynamic Copula Approach
to Recovering the Index Implied Volatility Skew. Journal of Financial Econometrics ,
10 (3), 457–493.

7 Volatility 139
Filipovi´c, D., Gourier, E., & Mancini, L. (2016). Quadratic Variance Swap Models.
Journal of Financial Economics , 119 (1), 44–68.
Fleming, J., Ostdiek, B., & Whaley, R. E. (1995). Predicting Stock Market Volatility:
AN e wM e a s u r e .Journal of Futures Markets , 15 (3), 265–302.
Flint, E., & Maré, E. (2017). Fractional Black-Scholes Option Pricing, Volatility
Calibration and Implied Hurst Exponents in South African Context. South African
Journal of Economic and Management Sciences , 20 (1), a1532.
Fu, X., Sandri, M., & Shackleton, M. B. (2016). Asymmetric Effects of Volatility Risk
on Stock Returns: Evidence from VIX and VIX Futures. Journal of Futures Markets ,
36 (11), 1029–1056.
Ge, W . (2016). A Survey of Three Derivative-Based Methods to Harvest the Volatility
Premium in Equity Markets. Journal of Investing , 25 (3), 48–58.
Gehricke, S. A., & Zhang, J. E. (2018). Modeling VXX. Journal of Futures Markets ,
38(8), 958–976.
Glasserman, P ., & Wu, Q. (2010). Forward and Future Implied Volatility.International
Journal of Theoretical and Applied Finance , 14 (3), 407–432.
Grasselli, M., & Wagalath, L. (2018). VIX vs VXX: A Joint Analytical Framework
(Working Paper). Available online: https://ssrn.com/abstract=3144526.
Hafner, R., & Wallmeier, M. (2007). Volatility as an Asset Class: European Evidence.
European Journal of Finance , 13(7), 621–644.
Hancock, G. D. (2013). VIX Futures ETNs: Three Dimensional Losers. Accounting
and Finance Research , 2(3), 53–64.
Härdle, W ., and Silyakova, E. (2010).Volatility Investing with Variance Swaps (Working
Paper). Available online: https://ssrn.com/abstract=2894245.
He, D. X., Hsu, J. C., & Rue, N. (2015). Option-Writing Strategies in a Low-Volatility
Framework. Journal of Investing , 24 (3), 116–128.
Husson, T ., & McCann, C. J. (2011). The VXX ETN and Volatility Exposure. PIABA
Bar Journal , 18(2), 235–252.
Jackwerth, J. C. (2000). Recovering Risk Aversion from Option Prices and Realized
Returns. Review of Financial Studies , 13(2), 433–451.
Jarrow, R., Kchia, Y., Larsson, M., & Protter, P . (2013). Discretely Sampled Vari-
ance and Volatility Swaps Versus Their Continuous Approximations. Finance and
Stochastics, 17 (2), 305–324.
Konstantinidi, E., & Skiadopoulos, G. (2016). How Does the Market Variance Risk
Premium Vary Over Time? Evidence from S&P 500 Variance Swap Investment
Returns. Journal of Banking & Finance , 62, 62–75.
Kozhan, R., Neuberger, A., & Schneider, P . (2013). The Skew Risk Premium in the
Equity Index Market. Review of Financial Studies , 26 (9), 2174–2203.
Lamoureux, C. G., & Lastrapes, W . (1993). Forecasting Stock Return Variance:
T owards Understanding Stochastic Implied Volatility. Review of Financial Studies ,
6 (2), 293–326.
Lee, H., Liao, T ., & T ung, P . (2017). Investors’ Heterogeneity in Beliefs, the VIX
Futures Basis, and S&P 500 Index Futures Returns. Journal of Futures Markets ,
37 (9), 939–960.

140 Z. Kakushadze and J. A. Serur
Leontsinis, S., & Alexander, C. (2016). Arithmetic Variance Swaps. Quantitative
Finance, 17 (4), 551–569.
Liu, Z. F ., & van der Heijden, T . (2016). Model-Free Risk-Neutral Moments and Proxies
(Working Paper). Available online: https://ssrn.com/abstract=2641559.
Liu, B., & Dash, S. (2012). Volatility ETFs and ETNs. Journal of T rading , 7 (1),
43–48.
Liu, F ., Pantelous, A. A., & von Mettenheim, H.-J. (2018). Forecasting and T rading
High Frequency Volatility on Large Indices. Quantitative Finance, 18(5), 737–748.
Liverance, E. (2010). Variance Swap. In R. Cont (Ed.), Encyclopedia of Quantitative
Finance. Hoboken, NJ: Wiley.
Maghrebi, N., Kim, M., & Nishina, K. (2007). The KOSPI200 Implied Volatility
Index: Evidence of Regime Shifts in Expected Volatility. Asia-Paciﬁc Journal of
Financial Studies , 36 (2), 163–187.
Martin, I. (2011). Simple Variance Swaps (Working Paper). Available online: http://
www.nber.org/papers/w16884.
Mayhew, S. (1995). Implied Volatility. Financial Analysts Journal , 51(4), 8–20.
Miao, G. J., Wei, B., & Zhou, H. (2012). Ambiguity Aversion and Variance Premium
(Working Paper). Available online: https://ssrn.com/abstract=2023765.
Mixon, S. (2007). The Implied Volatility T erm Structure of Stock Index Options.
Journal of Empirical Finance , 14 (3), 333–354.
Mixon, S. (2011). What Does Implied Volatility Skew Measure? Journal of Derivatives ,
18(4), 9–25.
Moran, M. T ., & Dash, S. (2007). VIX Futures and Options: Pricing and Using
Volatility Products to Manage Downside Risk and Improve Efﬁciency in Equity
Portfolios. Journal of T rading, 2(3), 96–105.
Nossman, M., & Wilhelmsson, A. (2009). Is the VIX Futures Market Able to Pre-
dict the VIX Index? A T est of the Expectation Hypothesis. Journal of Alternative
Investments, 12(2), 54–67.
Prokopczuk, M., & Simen, C. W . (2014). The Importance of the Volatility Risk
Premium for Volatility Forecasting. Journal of Banking & Finance , 40, 303–320.
Rujivan, S., & Zhu, S. P . (2012). A Simpliﬁed Analytical Approach for Pricing Dis-
cretely Sampled Variance Swaps with Stochastic Volatility. Applied Mathematics
Letters, 25 (11), 1644–1650.
Saretto, A., & Goyal, A. (2009). Cross-Section of Option Returns and Volatility.
Journal of Financial Economics , 94 (2), 310–326.
Schoutens, W . (2005). Moment Swaps. Quantitative Finance , 5 (6), 525–530.
Shaikh, I., & Padhi, P . (2015). The Implied Volatility Index: Is ‘Investor Fear Gauge’
or ‘Forward-Looking’?Borsa Istanbul Review , 15 (1), 44–52.
Simon, D. P ., & Campasano, J. (2014). The VIX Futures Basis: Evidence and T rading
Strategies. Journal of Derivatives , 21(3), 54–69.
Siriopoulos, C., & Fassas, A. (2009). Implied Volatility Indices—A Review (Working
Paper). Available online: https://ssrn.com/abstract=1421202.
Skiadopoulos, G. (2004). The Greek Implied Volatility Index: Construction and Prop-
erties. Applied Financial Economics , 14 (16), 1187–1196.

7 Volatility 141
Skiadopoulos, G., Hodges, S., & Clewlow, L. (1999). The Dynamics of the S&P 500
Implied Volatility Surface. Review of Derivatives Research , 3(3), 263–282.
T odorov, V . (2010). Variance Risk-Premium Dynamics: The Role of Jumps. Review of
Financial Studies , 23(1), 345–383.
Whaley, R. E. (2000). The Investor Fear Gauge. Journal of Portfolio Management ,
26 (3), 12–16.
Whaley, R. E. (2009). Understanding the VIX. Journal of Portfolio Management , 35 (3),
98–105.
Wystup, U., & Zhou, Q. (2014). Volatility as Investment—Crash Protection with
Calendar Spreads of Variance Swaps. Journal of Applied Operational Research , 6 (4),
243–254.
Zhang, L. (2014). A Closed-form Pricing Formula for Variance Swaps with Mean-
Reverting Gaussian Volatility. ANZIAM Journal , 55 (4), 362–382.
Zhang, J. E., & Xiang, Y. (2008). The Implied Volatility Smirk. Quantitative Finance,
8(3), 263–284.
Zhang, J. E., & Zhu, Y. (2006). VIX Futures. Journal of Futures Markets , 26 (6),
521–531.
Zhang, J. E., Shu, J., & Brenner, M. (2010). The New Market for Volatility T rading.
Journal of Futures Markets , 30 (9), 809–833.
Zheng, W ., & Kwok, Y. K. (2014). Closed form Pricing Formulas for Discretely
Sampled Generalized Variance Swaps. Mathematical Finance , 24 (4), 855–881.
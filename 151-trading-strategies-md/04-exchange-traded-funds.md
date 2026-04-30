# Chapter 4: Exchange-Traded Funds (ETFs)

4
Exchange-T raded Funds (ETFs)
4.1 Strategy: Sector Momentum Rotation
Empirical evidence suggests that the momentum effect exists not only for
individual stocks but also for sectors and industries. 1 A sector momentum
rotation strategy is based on overweighing holdings in outperforming sectors
and underweighing holdings in underperforming sectors, where the “outper-
formance” and “underperformance” are based on momentum during the past
T -month formation period (which typically ranges from 6 to 12 months).
ETFs concentrated in speciﬁc sectors/industries offer a simple way to imple-
ment sector/industry rotation without having to buy or sell a large number
of underlying stocks. Similarly to Sect. 3.1, as a measure of sector/industry
momentum, we can use the corresponding ETF’s cumulative return:
R
cum
i (t ) = Pi (t )
Pi (t + T ) − 1 (4.1)
Here, Pi (t ) is the price of the ETF labeled by i . (As above, t + T is T months
in the past w.r.t.t .) Right after time t , the trader can, e.g., buy the ETFs in the
top decile by Rcum
i (t ) and hold the portfolio for a holding period (typically 1
to 3 months). Dollar-neutral strategies can also be constructed by, e.g., buying
1For some pertinent literature, see, e.g., Cavaglia and Vadim ( 2002), Conover et al. ( 2008), Doeswijk
and Vliet ( 2011), Dolvin and Kirby ( 2011), Gao and Ren ( 2015), Hong et al. ( 2007), Levis and Liodakis
(1999), Moskowitz and Grinblatt ( 1999), O’Neal ( 2000), Sefton and Scowcroft ( 2005), Simpson and
Grossman ( 2016), Sorensen and Burke ( 1986), Stovall ( 1996), Swinkels ( 2002), Szakmary and Zhou
(2015), Wang et al. ( 2017).
© The Author(s) 2018
Z. Kakushadze and J. A. Serur, 151 T rading Strategies,
https://doi.org/10.1007/978-3-030-02792-6_4
87

88 Z. Kakushadze and J. A. Serur
ETFs in the top decile and shorting ETFs in the bottom decile (as stocks, ETFs
can be shorted). 2
Strategy: Sector Momentum Rotation with MA filter
This is a variation/reﬁnement of the sector momentum rotation strategy.
An ETF in the top (bottom) decile is bought (sold) only if it passes an additional
ﬁlter based on a moving average MA (T
′) of this ETF’s price:
Rule =
{
Buy top-decile ETFs only if P > MA(T ′)
Short bottom-decile ETFs only if P < MA(T ′) (4.2)
Here, P is the ETF’s price at the time of the transaction, and MA (T ′) is
computed using daily prices ( T ′ can but need not be equal T ; e.g., T ′ can be
100–200 days).
Strategy: Dual-Momentum Sector Rotation
In long-only strategies, to mitigate the risk of buying ETFs when the broad
market is trending down, relative (i.e., cross-sectional) momentum of sector
ETFs can be augmented by the absolute (i.e., time-series) momentum of, e.g., a
broad market index ETF (see, e.g., Antonacci 2014, 2017).3 So, a long position
based on the sector rotation signal (discussed above) is established only if the
broad market index has an upward trend; otherwise, the total available funds
are invested into an ETF (e.g., gold or T reasury ETF) uncorrelated with the
broad market index:
Rule =
{
Buy top-decile ETFs if P > MA(T
′)
Buy an uncorrelated ETF if P ≤ MA(T ′) (4.3)
Here, P is the broad market index ETF’s price at the time of the transaction,
and MA(T ′) is the moving average of this ETF’s price. T ypically,T ′ is 100–200
days.
2For some literature on ETFs, see, e.g., Agapova ( 2011), Aldridge ( 2016), Ben-David et al. ( 2017),
Bhattacharya et al. ( 2017), Buetow and Henderson ( 2012), Clifford et al. ( 2014), Hill et al. ( 2015),
Krause et al. ( 2014), Madhavan ( 2016), Madura and Ngo ( 2008), Nyaradi ( 2010), Oztekin et al. ( 2017).
3For some additional literature on relative momentum, absolute momentum and related topics, see, e.g.,
Ahn et al. ( 2003), Bandarchuk and Hilscher ( 2013), Berk et al. ( 1999), Cooper et al. ( 2004), Fama and
French (2008), Hurst et al. ( 2017), Johnson ( 2002), Liu and Zhang ( 2008), Moskowitz et al. ( 2012), Sagi
and Seasholes ( 2007), Schwert ( 2003), Zhang ( 2006).

4 Exchange-T raded Funds (ETFs) 89
4.2 Strategy: Alpha Rotation
This is the same as the sector momentum rotation strategy with the cumulative
ETF returns Rcum
i replaced by ETF alphas αi , which are the regression coefﬁ-
cients corresponding to the intercept in a serial regression of the ETF returns 4
Ri (t ) over, e.g., the 3 Fama–French factors MKT (t ),S M B(t ), HML (t ) (see
fn. 16 in Sect. 3.7)5:
Ri (t ) = αi + β1,i MKT(t ) + β2,i SMB(t ) + β3,i HML(t ) + ϵi (t )( 4.4)
4.3 Strategy: R-squared
Empirical studies for mutual funds (see, e.g., Amihud and Goyenko 2013;
Ferson and Mo 2016) and ETFs (see, e.g., Garyn-T al 2014a, b) suggest
that augmenting alpha by an indicator based on R2 of a serial regression
of the returns Ri (t ) over multiple factors, e.g., the 3 Fama–French factors
MKT(t ),S M B(t ), HML (t ) plus Carhart’s momentum factor MOM (t ) (see
fn. 16 in Sect. 3.7), adds value in forecasting future returns. Thus, from the
serial regression
Ri (t ) = αi + β1,i MKT(t ) + β2,i SMB(t ) + β3,i HML(t )
+ β4,i MOM(t ) + ϵi (t )( 4.5)
we can estimate αi (the regression coefﬁcients corresponding to the intercept)
and the regression R2, which is deﬁned as (“SS” stands for “sum of squares”):
R2 = 1 − SSres
SStot
(4.6)
SSres =
N∑
i =1
ϵ2
i (t )( 4.7)
4T ypically, the estimation period is 1 year, and Ri (t ) are daily or weekly returns.
5Alpha here is Jensen’s alpha deﬁned for ETF returns as opposed to mutual fund returns as in Jensen
(1968). For some additional literature related to Jensen’s alpha, see, e.g., Bollen and Busse ( 2005), Droms
and Walker ( 2001), Elton et al. ( 1996), Goetzmann and Ibbotson ( 1994), Grinblatt and Titman ( 1992),
Jan and Hung ( 2004).

90 Z. Kakushadze and J. A. Serur
SStot =
N∑
i =1
(Ri (t ) − R(t ))2 (4.8)
R(t ) = 1
N
N∑
i =1
Ri (t )( 4.9)
An R-squared strategy then amounts to overweighing ETFs with higher “selec-
tivity” (deﬁned as 1 − R2 [Amihud and Goyenko 2013]) and underweighing
ETFs with lower “selectivity”. E.g., one can ﬁrst sort ETFs into quintiles by R2,
and then sort ETFs in each such quintile into further sub-quintiles by alpha
(resulting in 25 groups of ETFs). One can then, e.g., buy ETFs in the group
corresponding to the lowest R2 quintile and its highest alpha sub-quintile and
sell ETFs in the group corresponding to the highest R2 quintile and its lowest
alpha sub-quintile. Other variations are possible. Finally, the estimation period
and the returns for R2 can be the same as in the alpha rotation strategy (see
Sect. 4.2 and fn. 4). However, longer estimation periods can be considered,
especially if Ri (t ) are monthly returns. 6
4.4 Strategy: Mean-reversion
One way (among myriad others) to construct a mean-reversion strategy for
ETFs is to use the Internal Bar Strength (IBS) based on the previous day’s close
PC ,h i g h PH and low PL prices7:
IBS = PC − PL
PH − PL
(4.10)
Note that IBS ranges from 0 to 1. 8 An ETF can be thought of as being “rich”
if its IBS is close to 1, and as “cheap” if its IBS is close to 0. Upon sorting a
universe of ETFs cross-sectionally by IBS, a dollar-neutral strategy can, e.g., be
constructed by selling ETFs in the top decile and buying ETFs in the bottom
6A l s o ,n o t et h a ti nA m i h u da n dG o y e n k o(2013) R2 is a measure of active management of a mutual
fund. In Garyn-T al ( 2014a, b) R2 is applied to actively managed ETFs. For some additional literature
on actively managed ETFs, see, e.g., Mackintosh ( 2017), Meziani ( 2015), Rompotis ( 2011a, b), Schizas
(2014), Sherrill and Upton ( 2018).
7See, e.g., Pagonidis ( 2014). For some additional related literature, see, e.g., Brown et al. ( 2018), Caginalp
et al. ( 2014), Chan ( 2013) ,D u n i se ta l .(2013), Lai et al. ( 2016), Levy and Lieberman ( 2013), Marshall
et al. ( 2013), Rudy et al. ( 2010), Schizas et al. ( 2011), Smith et al. ( 2015), Yu and Webb ( 2014).
8An equivalent but more symmetrical measure is Y = IBS − 1/2 = (PC − P∗)/(PH − PL ),w h e r e
P∗ = (PH + PL )/2.N o t et h a tY ranges from 1/2 for PC = PH to −1/2 for PC = PL .

4 Exchange-T raded Funds (ETFs) 91
decile. As with stock strategies discussed above, weights can be uniform for all
long and all short ETFs, respectively, or nonuniform, e.g., based on historical
ETF volatilities. Furthermore, mean-reversion strategies we discussed above
for stocks can also be adapted to ETFs.
4.5 Strategy: Leveraged ETFs (LETFs)
A leveraged (inverse) ETF seeks to double or triple (the inverse of ) the daily
return of its underlying index. 9 T o maintain a daily leverage of 2× or 3×,
LETFs rebalance every day, which requires buying on the days when the market
is up and selling when the market is down. This can result in a negative drift
in the long term, which can be exploited by shorting both a leveraged ETF
and a leveraged inverse ETF (both with the same leverage and for the same
underlying index) and investing the proceeds into, e.g., a T reasury ETF. This
strategy can have a signiﬁcant downside risk in the short term if one of the
short ETF legs has a sizable positive return.
4.6 Strategy: Multi-asset T rend Following
One allure of ETFs is their diversiﬁcation power: ETFs allow to gain exposure
to different sectors, countries, asset classes, factors, etc., by taking positions in
a relatively small number of ETFs (as opposed to taking positions in a large
number of underlying instruments, e.g., thousands of stocks). Here, we focus
on long-only trend-following portfolios. One needs to determine the weight
w
i of each ETF. One (but by far not the only) way to ﬁx these weights is
as follows. First, as in the sector momentum rotation strategy, we compute
cumulative returns Rcum
i (over some period T , e.g., 6–12 months). We only
take ETFs with positive Rcum
i . If desired, optionally, we can further ﬁlter
out ETFs as in the sector momentum rotation strategy with an MA ﬁlter,
by keeping only the ETFs whose last closing prices Pi are higher than their
corresponding long-term moving averages MA i (T ′) (typically, the MA length
T ′ is 100–200 days). Now, instead of taking ETFs in the top decile by Rcum
i
(as in the sector momentum rotation strategy), we can assign nonzero weights
9For some literature on leveraged ETFs, see e.g., Avellaneda and Zhang ( 2010), Bai et al. ( 2015), Charupat
and Miu ( 2011), Cheng and Madhavan ( 2010), Ivanov and Lenkey ( 2014), Jarrow ( 2010), Jiang and
Peterburgsky ( 2017), Lu et al. ( 2012), Shum et al. ( 2016), T ang and Xu ( 2013), T rainor (2010), T uzun
(2013).

92 Z. Kakushadze and J. A. Serur
wi to all remaining ETFs, whose number in this context is relatively small to
begin with by design. The weights can, e.g., be assigned as follows:
wi = γ1 Rcum
i (4.11)
wi = γ2 Rcum
i /σi (4.12)
wi = γ3 Rcum
i /σ2
i (4.13)
Here: σi is the historical volatility; and the overall normalization coefﬁ-
cients γ1,γ2,γ 3 in each case are computed based on the requirement that∑ N
i =1 wi = 1 (where N is the number of ETFs in our portfolio after all ﬁlters
are applied, i.e., those with nonzero weights). Thus, the weights in Eq. ( 4.11)
are simply proportional to the past cumulative returns Rcum
i , which are taken
as the measure of momentum, so the expected returns are also given by (or,
more precisely, proportional to) R
cum
i . The issue with this weighting scheme
is that it overweighs volatile ETFs as on average Rcum
i ∝ σi . The weights
in Eq. ( 4.12) mitigate this, while the weights in Eq. ( 4.13) actually optimize
the Sharpe ratio of the ETF portfolio assuming a diagonal covariance matrix
Cij = diag(σ2
i ) for the ETF returns, i.e., by ignoring their correlations. 10
Imposing bounds wi ≤ wmax
i can further mitigate overweighing.
References
Agapova, A. (2011). Conventional Mutual Funds Versus Exchange-T raded Funds.
Journal of Financial Markets , 14 (2), 323–343.
Ahn, D.-H., Conrad, J., & Dittmar, R. (2003). Risk Adjustment and T rading Strate-
gies. Review of Financial Studies , 16 (2), 459–485.
Aldridge, I. (2016). ETFs, High-Frequency T rading, and Flash Crashes. Journal of
Portfolio Management , 43(1), 17–28.
Amihud, Y., & Goyenko, R. (2013). Mutual Fund’s R2 as Predictor of Performance.
Review of Financial Studies , 26 (3), 667–694.
Antonacci, G. (2014). Dual Momentum Investing: An Innovative Strategy for Higher
Returns with Lower Risk .N e wY o r k ,N Y :M c G r a w - H i l l .
Antonacci, G. (2017). Risk Premia Harvesting Through Dual Momentum. Journal of
Management & Entrepreneurship , 11(1), 27–55.
Avellaneda, M., & Zhang, S. (2010). Path-Dependence of Leveraged ETF Returns.
Journal on Financial Mathematics , 1(1), 586–603.
10For some literature on multi-asset portfolios, dynamic asset allocation and related topics, see, e.g.,
Bekkers et al. ( 2009), Black and Litterman ( 1992), Detemple and Rindisbacher ( 2010), Doeswijk et al.
(2014), Faber ( 2015, 2016), Mladina ( 2014), Petre ( 2015), Sassetti and T ani ( 2006), Sharpe ( 2009),
Sharpe and Perold ( 2009), Sørensen ( 1999), T ripathi and Garg ( 2016), Wu ( 2003), Zakamulin ( 2014).

4 Exchange-T raded Funds (ETFs) 93
Bai, Q., Bond, S. A., & Hatch, B. C. (2015). The Impact of Leveraged and Inverse
ETFs on Underlying Real Estate Returns. Real Estate Economics , 43(1), 37–66.
Bandarchuk, P ., & Hilscher, J. (2013). Sources of Momentum Proﬁts: Evidence on
the Irrelevance of Characteristics. Review of Finance , 17 (2), 809–845.
Bekkers, N., Doeswijk, R. Q., & Lam, T . W . (2009). Strategic Asset Allocation: Deter-
mining the Optimal Portfolio withT en Asset Classes. Journal ofWealth Management,
12(3), 61–77.
Ben-David, I., Franzoni, F . A., & Moussawi, R. (2017). Do ETFs Increase
Volatility? Journal of Finance (forthcoming). Available online: https://ssrn.com/
abstract=1967599.
Berk, J., Green, R., & Naik, V . (1999). Optimal Investment, Growth Options and
Security Returns. Journal of Finance , 54 (5), 1153–1608.
Bhattacharya, U., Loos, B., Meyer, S., & Hackethal, A. (2017). Abusing ETFs. Review
of Finance , 21(3), 1217–1250.
Black, F ., & Litterman, R. (1992). Global Portfolio Optimization. Financial Analysts
Journal, 48(5), 28–43.
Bollen, N. P . B., & Busse, J. A. (2005). Short-T erm Persistence in Mutual Fund
Performance. Review of Financial Studies , 18(2), 569–597.
Brown, D. C., Davies, S., & Ringgenberg, M. (2018). ETF Arbitrage and Return Pre-
dictability (Working Paper). Available online: https://ssrn.com/abstract=2872414.
Buetow, G. W ., & Henderson, B. J. (2012). An Empirical Analysis of Exchange-T raded
Funds. Journal of Portfolio Management , 38(4), 112–127.
Caginalp, G., DeSantis, M., & Sayrak, A. (2014). The Nonlinear Price Dynamics of
US Equity ETFs. Journal of Econometrics , 183(2), 193–201.
Cavaglia, S., & Vadim, M. (2002). Cross-Industry, Cross Country Allocation. Finan-
cial Analysts Journal , 58(6), 78–97.
Chan, E. P . (2013).Algorithmic T rading: Winning Strategies and Their Rationale. Hobo-
ken, NJ: Wiley.
Charupat, N., & Miu, P . (2011). The Pricing and Performance of Leveraged Exchange-
T raded Funds. Journal of Banking & Finance , 35 (4), 966–977.
Cheng, M., & Madhavan, A. (2010). The Dynamics of Leveraged and Inverse
Exchange-T raded Funds.Journal of Investment Management , 7 (4), 43–62.
Clifford, C. P ., Fulkerson, J. A., & Jordan, B. D. (2014). What Drives ETF Flows?
Financial Review , 49 (3), 619–642.
Conover, C. M., Jensen, G., Johnson, R., & Mercer, M. (2008). Sector Rotation and
Monetary Conditions. Journal of Investing , 28(1), 34–46.
Cooper, M. J., Gutierrez, R. C., Jr., & Hameed, A. (2004). Market States and Momen-
tum. Journal of Finance , 59 (3), 1345–1365.
Detemple, J., & Rindisbacher, M. (2010). Dynamic Asset Allocation: Portfolio
Decomposition Formula and Applications. Review of Financial Studies , 23(1), 25–
100.
Doeswijk, R., Lam, T ., & Swinkels, L. (2014). The Global Multi-asset Market Port-
folio, 1959–2012. Financial Analysts Journal , 70 (2), 26–41.

94 Z. Kakushadze and J. A. Serur
Doeswijk, R., & van Vliet, P . (2011). Global T actical Sector Allocation: A Quantitative
Approach. Journal of Portfolio Management , 28(1), 29–47.
Dolvin, S., & Kirby, J. (2011). Momentum T rading in Sector ETFs. Journal of Index
Investing, 2(3), 50–57.
Droms, W . G., & Walker, D. A. (2001). Performance Persistence of International
Mutual Funds. Global Finance Journal , 12(2), 237–248.
Dunis, C., Laws, J., & Rudy, J. (2013, October). Mean Reversion Based on Autocor-
relation: A Comparison Using the S&P 100 Constituent Stocks and the 100 Most
Liquid ETFs. ETF Risk , pp. 36–41.
Elton, E. J., Gruber, M. J., & Blake, C. R. (1996). The Persistence of Risk-Adjusted
Mutual Fund Performance. Journal of Business , 69 (2), 133–157.
Faber, M. (2015). Learning to Play Offense and Defense: Combining Value and Momen-
tum from the Bottom Up, and the T op Down (Working Paper). Available online:
https://ssrn.com/abstract=2669202.
Faber, M. (2016).The T rinity Portfolio: A Long-T erm Investing Framework Engineered for
Simplicity, Safety, and Outperformance (Working Paper). Available online: https://
ssrn.com/abstract=2801856.
Fama, E. F ., & French, K. R. (2008). Dissecting Anomalies. Journal of Finance , 63(4),
1653–1678.
Ferson, W ., & Mo, H. (2016). Performance Measurement with Selectivity, Market
and Volatility Timing. Journal of Financial Economics , 121(1), 93–110.
Gao, B., & Ren, R.-E. (2015). A New Sector Rotation Strategy and Its Performance
Evaluation: Based on a Principal Component Regression Model (Working Paper).
Available online: https://ssrn.com/abstract=2628058.
Garyn-T al, S. (2014a). An Investment Strategy in Active ETFs. Journal of Index Invest-
ing, 4 (1), 12–22.
Garyn-T al, S. (2014b). Explaining and Predicting ETFs Alphas: The R2 Methodology.
Journal of Index Investing , 4 (4), 19–32.
Goetzmann, W . N., & Ibbotson, R. G. (1994). Do Winners Repeat? Journal of Portfolio
Management, 20 (2), 9–18.
Grinblatt, M., & Titman, S. (1992). The Persistence of Mutual Fund Performance.
Journal of Finance , 47 (5), 1977–1984.
Hill, J. M., Nadig, D., & Hougan, M. (2015). A Comprehensive Guide to Exchange-
T raded Funds (ETFs). Research Foundation Publications , 2015 (3), 1–181.
Hong, H., T orous, W ., & Valkanov, R. (2007). Do Industries Lead Stock Markets?
Journal of Financial Economics , 83(2), 367–396.
Hurst, B., Ooi, Y. H., & Pedersen, L. H. (2017). A Century of Evidence on T rend-
Following Investing. Journal of Portfolio Management , 44 (1), 15–29.
Ivanov, I.T ., & Lenkey, S. L. (2014). Are Concerns About Leveraged ETFs Overblown?
Finance and Economics Discussion Series (FEDS) , Paper No. 2014-106. Washington,
DC: Board of Governors of the Federal Reserve System. Available online: https://
www.federalreserve.gov/econresdata/feds/2014/ﬁles/2014106pap.pdf.
Jan, T . C., & Hung, M. W . (2004). Short-Run and Long-Run Persistence in Mutual
Funds. Journal of Investing , 13(1), 67–71.

4 Exchange-T raded Funds (ETFs) 95
Jarrow, R. A. (2010). Understanding the Risk of Leveraged ETFs. Finance Research
Letters, 7 (3), 135–139.
Jensen, M. C. (1968). The Performance of Mutual Funds in the Period 1945–1964.
Journal of Finance , 23(2), 389–416.
Jiang, X., & Peterburgsky, S. (2017). Investment Performance of Shorted Leveraged
ETF Pairs. Applied Economics , 49 (44), 4410–4427.
Johnson, T . C. (2002). Rational Momentum Effects. Journal of Finance , 57 (2), 585–
608.
Krause, T ., Ehsani, S., & Lien, D. (2014). Exchange-T raded Funds, Liquidity and
Volatility.Applied Financial Economics , 24 (24), 1617–1630.
Lai, H.-C., Tseng, T .-C., & Huang, S.-C. (2016). Combining Value Averaging and
Bollinger Band for an ETF T rading Strategy. Applied Economics , 48(37), 3550–
3557.
Levis, M., & Liodakis, M. (1999). The Proﬁtability of Style Rotation Strategies in the
United Kingdom. Journal of Portfolio Management , 26 (1), 73–86.
Levy, A., & Lieberman, O. (2013). Overreaction of Country ETFs to US Market
Returns: Intraday vs. Daily Horizons and the Role of Synchronized T rading. Journal
of Banking & Finance , 37 (5), 1412–1421.
Liu, L. X., & Zhang, L. (2008). Momentum Proﬁts, Factor Pricing, and Macroeco-
nomic Risk. Review of Financial Studies , 21(6), 2417–2448.
Lu, L., Wang, J., & Zhang, G. (2012). Long T erm Performance of Leveraged ETFs.
Financial Services Review , 21(1), 63–80.
Mackintosh, P . (2017). It’s All About Active ETFs. Journal of Index Investing , 7 (4),
6–15.
Madhavan, A. N. (2016). Exchange-T raded Funds and the New Dynamics of Investing .
Oxford, UK: Oxford University Press.
Madura, J., & Ngo, T . (2008). Impact of ETF Inception on the Valuation and T rading
of Component Stocks. Applied Financial Economics , 18(12), 995–1007.
Marshall, B. R., Nguyen, N. H., & Visaltanachoti, N. (2013). ETF Arbitrage: Intraday
Evidence. Journal of Banking & Finance , 37 (9), 3486–3498.
Meziani, A. S. (2015). Active Exchange-T raded Funds: Are We There Yet? Journal of
Index Investing, 6 (2), 86–98.
Mladina, P . (2014). Dynamic Asset Allocation with Horizon Risk: Revisiting Glide
Path Construction. Journal of Wealth Management , 16 (4), 18–26.
Moskowitz, T . J., & Grinblatt, M. (1999). Do Industries Explain Momentum? Journal
of Finance , 54 (4), 1249–1290.
Moskowitz, T . J., Ooi, Y. H., & Pedersen, L. H. (2012). Time Series Momentum.
Journal of Financial Economics , 104 (2), 228–250.
Nyaradi, J. (2010). Super Sectors: How to Outsmart the Market Using Sector Rotation
and ETFs . Hoboken, NJ: Wiley.
O’Neal, E. S. (2000). Industry Momentum and Sector Mutual Funds. Financial Ana-
lysts Journal , 56 (4), 37–49.

96 Z. Kakushadze and J. A. Serur
Oztekin, A. S., Mishra, S., Jain, P . K., Daigler, R. T ., Strobl, S., & Holowczak, R. D.
(2017). Price Discovery and Liquidity Characteristics for U.S. Electronic Futures
and ETF Markets. Journal of T rading, 12(2), 59–72.
Pagonidis, A. S. (2014). The IBS Effect: Mean Reversion in Equity ETFs (Working
Paper). Available online: http://www.naaim.org/wp-content/uploads/2014/04/
00V_Alexander_Pagonidis_The-IBS-Effect-Mean-Reversion-in-Equity-ETFs-1.
pdf .
Petre, G. (2015). A Case for Dynamic Asset Allocation for Long T erm Investors.
Procedia Economics and Finance , 29, 41–55.
Rompotis, G. G. (2011a). The Performance of Actively Managed Exchange T raded
Funds. Journal of Index Investing , 1(4), 53–65.
Rompotis, G. G. (2011b). Active vs. Passive Management: New Evidence from
Exchange T raded Funds. International Review of Applied Financial Issues and Eco-
nomics, 3(1), 169–186.
Rudy, J., Dunis, C., & Laws, J. (2010). Proﬁtable Pair T rading: A Comparison Using
the S&P 100 Constituent Stocks and the 100 Most Liquid ETFs (Working Paper).
Available online: https://ssrn.com/abstract=2272791.
Sagi, J., & Seasholes, M. (2007). Firm-Speciﬁc Attributes and the Cross-Section of
Momentum. Journal of Financial Economics , 84 (2), 389–434.
Sassetti, P ., & T ani, M. (2006). Dynamic Asset Allocation Using Systematic Sector
Rotation. Journal of Wealth Management , 8(4), 59–70.
Schizas, P . (2014). Active ETFs and Their Performance vis-à-vis Passive ETFs, Mutual
Funds, and Hedge Funds. Journal of Wealth Management , 17 (3), 84–98.
Schizas, P ., Thomakos, D. D., & Wang, T . (2011). Pairs T rading on International ETFs
(Working Paper). Available online: https://ssrn.com/abstract=1958546.
Schwert, G. W . (2003). Anomalies and market efﬁciency. In G. M. Constantinides,
M. Harris, & R. M. Stulz (Eds.), Handbook of the Economics of Finance, V ol 1B.
(1st ed., Chapter 15, pp. 939–974). Amsterdam, The Netherlands: Elsevier.
Sefton, J. A., & Scowcroft, A. (2005). Understanding Momentum. Financial Analysts
Journal, 61(2), 64–82.
Sharpe, W . F . (2009). Adaptive Asset Allocation Policies. Financial Analysts Journal ,
66 (3), 45–59.
Sharpe, W . F ., & Perold, A. F . (1988). Dynamic Strategies for Asset Allocation. Finan-
cial Analysts Journal , 44 (1), 16–27.
Sherrill, D. E., & Upton, K. (2018). Actively Managed ETFs vs Actively Managed
Mutual Funds. Managerial Finance , 44 (3), 303–325.
Shum, P ., Hejazi, W ., Haryanto, E., & Rodier, A. (2016). Intraday Share Price Volatility
and Leveraged ETF Rebalancing. Review of Finance , 20 (6), 2379–2409.
Simpson, M. W ., & Grossman, A. (2016). The Role of Industry Effects in Simulta-
neous Reversal and Momentum Patterns in One-Month Stock Returns. Journal of
Behavioral Finance , 17 (4), 309–320.
Smith, D. M., & Pantilei, V . S. (2015). Do “Dogs of the World” Bark or Bite? Evidence
from Single-Country ETFs. Journal of Investing , 24 (1), 7–15.

4 Exchange-T raded Funds (ETFs) 97
Sørensen, C. (1999). Dynamic Asset Allocation and Fixed Income Management.
Journal of Financial and Quantitative Analysis , 34 (4), 513–531.
Sorensen, E. H., & Burke, T . (1986). Portfolio Returns from Active Industry Group
Rotation. Financial Analysts Journal , 42(5), 43–50.
Stovall, S. (1996). Sector Investing .N e wY o r k ,N Y :M c G r a wH i l lI n c .
Swinkels, L. (2002). International Industry Momentum. Journal of Asset Management ,
3(2), 124–141.
Szakmary, A. C., & Zhou, X. (2015). Industry Momentum in an Earlier Time: Evi-
dence from the Cowles Data. Journal of Financial Research , 38(3), 319–347.
T ang, H., & Xu, X. E. (2013). Solving the Return Deviation Conundrum of Leveraged
Exchange-T raded Funds. Journal of Financial and Quantitative Analysis , 48(1),
309–342.
T rainor, W . J., Jr. (2010). Do Leveraged ETFs Increase Volatility? T echnology and
Investment, 1(3), 215–220.
T ripathi, V ., & Garg, S. (2016). A Cross-Country Analysis of Pricing Efﬁciency of
Exchange T raded Funds. Journal of Applied Finance , 22(3), 41–63.
T uzun, T . (2013). Are Leveraged and Inverse ETFs the New Portfolio Insurers? Finance
and Economics Discussion Series (FEDS) , Paper No. 2013-48. Washington, DC:
Board of Governors of the Federal Reserve System. Available online: https://www.
federalreserve.gov/pubs/feds/2013/201348/201348pap.pdf.
Wang, J., Brooks, R., Lu, X., & Holzhauer, H. M. (2017). Sector Momentum. Journal
of Investing , 26 (2), 48–60.
Wu, L. (2003). Jumps and Dynamic Asset Allocation. Review of Quantitative Finance
and Accounting , 20 (3), 207–243.
Yu, S., & Webb, G. (2014). The Proﬁtability of Pairs T rading Strategies Based on ETFs
(Working Paper). Available online: http://swfa2015.uno.edu/B_Asset_Pricing_III/
paper_196.pdf .
Zakamulin, V . (2014). Dynamic Asset Allocation Strategies Based on Unexpected
Volatility.Journal of Alternative Investments , 16 (4), 37–50.
Zhang, X. F . (2006). Information Uncertainty and Stock Returns. Journal of Finance ,
61(1), 105–136.
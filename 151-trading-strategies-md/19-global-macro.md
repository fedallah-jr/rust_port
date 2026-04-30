# Chapter 19: Global Macro

19
Global Macro
19.1 Generalities
Actually, macro trading strategies constitute an investment style, not an asset
class. These types of strategies are not limited to any particular asset class or
a geographical region and can invest in stocks, bonds, currencies, commodi-
ties, derivatives, etc., seeking to capitalize on regional, economic and political
changes around the world. While many macro strategies are based on analysts’
subjective opinions (these are discretionary strategies), a systematic approach
(non-discretionary strategies) also plays a prominent role. Global macro strate-
gies can vary by their style, e.g., there are directional strategies, long-short
strategies, relative value strategies, etc.
1
19.2 Strategy: Fundamental Macro Momentum
This strategy aims to capture returns from the market underreaction to
changes in macroeconomic trends by buying (selling) assets favored (adversely
affected) by incoming macroeconomic trends. Different asset classes can be
used in building an investment portfolio, e.g., global equity indexes, currencies,
1Macro strategies can be divided into 3 classes: discretionary macro, systematic macro, and CTA/managed
futures. For some literature on macro strategies and related topics, see, e.g., Asgharian et al. ( 2004), Chung
(2000), Connor and Woo ( 2004), Dobson ( 1984), Drobny ( 2006), Fabozzi et al. ( 2010), Fung and Hsieh
(1999), Gliner ( 2014), Kidd ( 2014), Lambert et al. ( 2006), Potjer and Gould ( 2007), Stefanini ( 2006),
Zaremba ( 2014).
© The Author(s) 2018
Z. Kakushadze and J. A. Serur, 151 T rading Strategies,
https://doi.org/10.1007/978-3-030-02792-6_19
263

264 Z. Kakushadze and J. A. Serur
government bonds, etc. 2 The “state variables” to consider are the business
cycle, international trade, monetary policy, and risk sentiment trends (see,
e.g., Brooks 2017).3 E.g., equity indexes from some number of countries are
ranked using the values of the aforesaid 4 state variables for each country. 4
A zero-cost portfolio can then be constructed by, e.g., going long the indexes
in the top decile and shorting those in the bottom decile. The so-constructed
portfolios for various asset classes can, e.g., be combined with equal weights.
Typically, the holding period ranges from three to six months.
19.3 Strategy: Global Macro Inflation Hedge
Exogenous shocks (such as a political or geopolitical issue) can have an impact
on commodity prices such as oil leading to an increase in prices in oil-dependent
economies. There are two steps in this process: (i) a pass-through from com-
modity prices to the headline inﬂation (HI), and (ii) then, a pass-through from
HI to the core inﬂation (CI). 5 I.e., HI quickly reﬂects various shocks around
the world. So, the global macro inﬂation hedge strategy is based on the spread
between HI and CI as an indicator to hedge inﬂation using commodities 6:
CA = max
(
0, min
( HIYo Y − CIYo Y
HIYo Y
, 1
))
(19.1)
Here: CA is the commodity allocation percentage within the portfolio, and
“YoY” stands for“year-on-year”. The hedge can be executed by, e.g., buying
a basket of various commodities through ETFs, futures, etc. (see, e.g., Fulli-
Lemaire 2013).
2Different asset classes are affected by the same macroeconomic trends differently. E.g., increasing growth
is positive for equities and currencies, but negative for bonds.
3Business cycle trends can be estimated using 1-yr changes in the real GDP growth and CPI inﬂation
forecast, each contributing with a 50% weight. International trade trends can be estimated using 1-yr
changes in spot FX rates against an export-weighted basket. Monetary policy trends can be estimated
using 1-yr changes in short-term rates. Risk sentiment trends can be estimated using 1-yr equity market
excess returns. For some literature on the rationale behind these variables, see, e.g., Bernanke and Kuttner
(2005), Clarida and Waldman ( 2007), Eichenbaum and Evans ( 1995).
4There is a variety of ways to do this ranking using the 4 variables. See, e.g., Sect. 3.6.
5HI is the raw inﬂation measured by indices such as the Consumer Price Index (CPI) based on prices
of goods and services in a broad basket, while CI excludes some products such as commodities, which
are highly volatile and add sizable noise to the index. For some pertinent literature, see, e.g., Blanchard
and Gali ( 2007), Blanchard and Riggi ( 2013), Clark and T erry ( 2010), Hamilton ( 2003), Marques et al.
(2003), T rehan (2005), van den Noord and André ( 2004).
6For some literature on using commodities as an inﬂation hedge, see, e.g., Amenc et al. ( 2009), Bodie
(1983), Bodie and Rosansky ( 1980), Greer ( 1978), Hoevenaars et al. ( 2008), Jensen et al. ( 2002).

19 Global Macro 265
19.4 Strategy: Global Fixed-Income Strategy
This systematic macro trading strategy is based on a cross-sectional analysis
of government bonds from various countries using variables such as (see, e.g.,
Brück and Fan 2017) GDP, inﬂation, sovereign risk, real interest rate, output
gap, value, momentum, term spread, and the so-called Cochrane-Piazzesi pre-
dictor (Cochrane and Piazzesi 2005). Thus, said bonds can be ranked based on
these factors and a zero-cost portfolio can be constructed by buying bonds in
the top quantile and selling bonds in the bottom quantile. Similarly to Sect. 3.6,
multifactor portfolios can also be constructed. Typically, country-bond ETFs
are used in such portfolios.
7
19.5 Strategy: T rading on Economic
Announcements
Empirical evidence suggests that stocks tend to yield higher returns on impor-
tant announcement dates such Federal Open Market Committee (FOMC)
announcements.
8 So, a simple macro trading strategy consists of buying stocks
on important announcement days (ADs), such as the FOMC announcements,
and switching to risk-free assets during non-announcement days (NDAs). This
is done via ETFs, futures, etc., as opposed to individual stocks, as the strat-
egy involves moving from 100% allocated in equities to 100% allocated in
T reasuries (see, e.g., Stotz 2016).9
References
Ai, H., & Bansal, R. (2016). Risk Preferences and the Macro Announcement Premium
(Working Paper). Available online: https://ssrn.com/abstract=2827445.
Amenc, N., Martellini, L., & Ziemann, V . (2009). Inﬂation-Hedging Properties of
Real Assets and Implications for Asset-Liability Management Decisions. Journal of
Portfolio Management , 35 (4), 94–110.
7For some literature on factor investing in ﬁxed-income assets, see, e.g., Beekhuizen et al. ( 2016), Correia
et al. ( 2012), Houweling and van Vundert ( 2017), Koijen et al. ( 2018), L ’Hoir and Boulhabel ( 2010),
Staal et al. ( 2015).
8For some pertinent literature, see, e.g., Ai and Bansal ( 2016), Bernanke and Kuttner ( 2005), Boyd et al.
(2005), Donninger ( 2015), Graham et al. ( 2003), Jones et al. ( 1998), Lucca and Moench ( 2012), Savor
and Wilson ( 2013).
9This strategy can be augmented with various (e.g., technical) ﬁlters (see, e.g., Stotz 2016).

266 Z. Kakushadze and J. A. Serur
Asgharian, M., Diz, F ., Gregoriou, G. N., & Rouah, F . (2004). The Global Macro
Hedge Fund Cemetery. Journal of Derivatives Accounting , 1(2), 187–194.
Beekhuizen, P ., Duyvesteyn, J., Martens, M., & Zomerdijk, C. (2016). Carry
Investing on the Yield Curve (Working Paper). Available online: http://ssrn.com/
abstract=2808327.
Bernanke, B. S., & Kuttner, K. N. (2005). What Explains the Stock Market’s Reaction
to Federal Reserve Policy? Journal of Finance , 60 (3), 1221–1257.
Blanchard, O. J., & Gali, J. (2007). The Macroeconomic Effects of Oil Shocks: Why Are
the 2000s so Different from the 1970s? (Working Paper). Available online: http://
www.nber.org/papers/w13368.pdf.
Blanchard, O. J., & Riggi, M. (2013). Why Are the 2000s so Different from the
1970s? A Structural Interpretation of Changes in the Macroeconomic Effects of
Oil Prices. Journal of the European Economic Association , 11(5), 1032–1052.
Bodie, Z. (1983). Commodity Futures as a Hedge against Inﬂation. Journal of Portfolio
Management, 9 (3), 12–17.
Bodie, Z., & Rosansky, V . I. (1980). Risk and Return in Commodity Futures. Financial
Analysts Journal , 36 (3), 27–39.
Boyd, J. H., Hu, J., & Jagannathan, R. (2005). The Stock Market’s Reaction to
Unemployment News: Why Bad News Is Usually Good for Stocks. Journal of
Finance, 60 (2), 649–672.
Brooks, J. (2017). A Half Century of Macro Momentum (Working Paper). Available
online: https://www.aqr.com/-/media/AQR/Documents/Insights/White-Papers/
A-Half-Century-of-Macro-Momentum.pdf .
Brück, E., & Fan, Y. (2017). Smart Beta in Global Government Bonds and Its Risk
Exposure (Working Paper). Available online: https://www.cfasociety.org/France/
Documents/QuantAwards2017_Etienne%20BRUECK%20and%20Yuanting
%20FAN_EDHEC.pdf.
Chung, S. Y. (2000). Review of Macro T rading and Investment Strategies: Macroeco-
nomic Arbitrage in Global Markets. Journal of Alternative Investments , 3(1), 84–85.
Clarida, R., & Waldman, D. (2007). Is Bad News About Inﬂation Good News for the
Exchange Rate? (Working Paper). Available online: http://www.nber.org/papers/
w13010.pdf .
Clark, T . E., & T erry, S. J. (2010). Time Variation in the Inﬂation Passthrough of
Energy Prices. Journal of Finance , 42(7), 1419–1433.
Cochrane, J. H., & Piazzesi, M. (2005). Bond Risk Premia. American Economic Review,
95 (1), 138–160.
Connor, G., & Woo, M. (2004). An Introduction to Hedge Funds (Working Paper).
Available online: http://eprints.lse.ac.uk/24675/1/dp477.pdf .
Correia, M. M., Richardson, S. A., & T una, A. I. (2012). Value Investing in Credit
Markets. Review of Accounting Studies , 17 (3), 572–609.
Dobson, M. W . R. (1984). Global Investment Portfolios: The United Kingdom and
Scandinavia. ICFA Continuing Education Series , 4, 56–60.

19 Global Macro 267
Donninger, C. (2015). T rading the Patience of Mrs. Yellen. A Short Vix-Futures Strategy
for FOMC Announcement Days (Working Paper). Available online: https://ssrn.
com/abstract=2544445.
Drobny, S. (2006). Inside the House of Money: T op Hedge Fund T raders on Proﬁting in
the Global Markets . Hoboken, NJ: Wiley.
Eichenbaum, M., & Evans, C. L. (1995). Some Empirical Evidence on the Effects
of Shocks to Monetary Policy on Exchange Rates. Quarterly Journal of Economics ,
110 (4), 975–1009.
Fabozzi, F . J., Focardi, S. M., & Jonas, C. (2010). Investment Management After the
Global Financial Crisis. Charlottesville, VA: The Research Foundation of CFA Insti-
tute.
Fulli-Lemaire, N. (2013). An Inﬂation Hedging Strategy with Commodities: A Core
Driven Global Macro. Journal of Investment Strategies , 2(3), 23–50.
Fung, W ., & Hsieh, D. A. (1999). A Primer on Hedge Funds. Journal of Empirical
Finance, 6 (3), 309–331.
Gliner, G. (2014). Global Macro T rading: Proﬁting in a New World Economy . Hoboken,
NJ: Wiley.
Graham, M., Nikkinen, J., & Sahlström, P . (2003). Relative Importance of Scheduled
Macroeconomic News for Stock Market Investors. Journal of Economics and Finance ,
27 (2), 153–165.
Greer, R. J. (1978). Conservative Commodities: A Key Inﬂation Hedge. Journal of
Portfolio Management , 4 (4), 26–29.
Hamilton, J. (2003). What Is an Oil Shock? Journal of Econometrics , 113(2), 363–398.
Hoevenaars, R. P . M. M., Molenaar, R. D. J., Schotman, P . C., & Steenkamp, T . B.
M. (2008). Strategic Asset Allocation with Liabilities: Beyond Stocks and Bonds.
Journal of Economic Dynamics and Control , 32(9), 2939–2970.
Houweling, P ., & van Vundert, J. (2017). Factor Investing in the Corporate Bond
Market. Financial Analysts Journal , 73(2), 100–115.
Jensen, G. R., Johnson, R. R., & Mercer, J. M. (2002). Tactical Asset Allocation and
Commodity Futures. Journal of Portfolio Management , 28(4), 100–111.
Jones, C. M., Lamont, O., & Lumsdaine, R. L. (1998). Macroeconomic News and
Bond Market Volatility. Journal of Financial Economics , 47 (3), 315–337.
Kidd, D. (2014). Global Tactical Asset Allocation: One Strategy Fits All? In Investment
Risk and Performance . Charlottesville, VA: CFA Institute.
Koijen, R. S. J., Moskowitz, T . J., Pedersen, L. H., & Vrugt, E. B. (2018). Carry.
Journal of Financial Economics , 127 (2), 197–225.
Lambert, M., Papageorgiou, N., & Platania, F . (2006). Market Efﬁciency and Hedge
Fund T rading Strategies (Working Paper). Available online: https://www.edhec.
edu/sites/www.edhec-portail.pprod.net/ﬁles/edhec_working_paper_market_
efﬁciency_and_hedge_fund_trading_strategies_f.compressed.pdf .
L ’Hoir, M., & Boulhabel, M. (2010). A Bond-Picking Model for Corporate Bond
Allocation. Journal of Portfolio Management , 36 (3), 131–139.
Lucca, D. O., & Moench, E. (2012). The Pre-FOMC Announcement Drift.
Journal
of Finance , 70 (1), 329–371.

268 Z. Kakushadze and J. A. Serur
Marques, C. R., Neves, P . D., & Sarmento, L. M. (2003). Evaluating Core Inﬂation
Indicators. Economic Modelling , 20 (4), 765–775.
Potjer, D., & Gould, C. (2007). Global T actical Asset Allocation: Exploiting the Oppor-
tunity of Relative Movements Across Asset Classes and Financial Markets .L o n d o n ,
UK: Risk Books.
Savor, P ., & Wilson, M. (2013). How Much Do Investors Care About Macroeconomic
Risk? Evidence from Scheduled Economic Announcements. Journal of Financial
and Quantitative Analysis , 48(2), 343–375.
Staal, A., Corsi, M., Shores, S., & Woida, C. (2015). A Factor Approach to Smart
Beta Development in Fixed Income. Journal of Index Investing , 6 (1), 98–110.
Stefanini, F . (2006). Investment Strategies of Hedge Funds . Chichester, UK: Wiley.
Stotz, O. (2016). Investment Strategies and Macroeconomic News Announcement
Days. Journal of Asset Management , 17 (1), 45–56.
T rehan, B. (2005). Oil Price Shocks and Inﬂation (Federal Reserve Bank of San Fran-
cisco, Economic Letter, No. 2005-28). Available online: https://www.frbsf.org/
economic-research/ﬁles/el2005-28.pdf .
van den Noord, P ., & André, C. (2007). Why Has Core Inﬂation Remained so Muted
in the Face of the Oil Shock? (Working Paper). Available online: https://doi.org/10.
1787/206408110285.
Zaremba, A. (2014). A Performance Evaluation Model for Global Macro Funds.
International Journal of Finance & Banking Studies , 3(1), 161–171.
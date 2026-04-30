# Chapter 15: Distressed Assets

15
Distressed Assets
15.1 Strategy: Buying and Holding
Distressed Debt
Distressed securities are those whose issuers are undergoing ﬁnancial/
operational distress, default or bankruptcy. One deﬁnition of distressed debt
is if the spread between the yields of T reasury bonds and those of the issuer
is greater than some preset number, e.g., 1000 basis points (see, e.g., Harner
2008). A common and simple distressed debt passive trading strategy amounts
to buying debt of a distressed company at a steep discount, 1 expecting (hoping)
that the company will repay its debt. T ypically, a distressed debt portfolio is
diversiﬁed across industries, entities and debt seniority level. It is anticipated
that only a small fraction of the held assets will have positive returns, but those
that do, will provide high rates of return (see, e.g., Greenhaus 1991). There are
two broad categories of passive distressed debt strategies (see, e.g., Altman and
Hotchkiss 2006). First, using various models (see Sect. 15.3) one can attempt
to predict whether a company will declare bankruptcy. Second, some strategies
focus on assets of companies in default or bankruptcy, a successful reorganiza-
tion being the driver of returns. T ypically, positions are established at key dates,
such as at the end of the default month or at the end of the bankruptcy-ﬁling
month, with the view of exploiting overreaction in the distressed debt market
(see, e.g., Eberhart and Sweeney 1992; Gilson 1995).
1For some pertinent literature, see, e.g., Altman ( 1998), Clark and Weinstein ( 1983), Eberhart et al.
(1999), Friewald et al. ( 2012), Gande et al. ( 2010), Gilson ( 2010, 2012), Harner ( 2011), Hotchkiss and
Mooradian ( 1997), Jiang et al. ( 2012), Lhabitant ( 2002), Morse and Shaw ( 1988), Moyer et al. ( 2012),
Putnam (1991), Quintero ( 1989), Reiss and Phelps ( 1991), Volpert ( 1991).
© The Author(s) 2018
Z. Kakushadze and J. A. Serur, 151 T rading Strategies,
https://doi.org/10.1007/978-3-030-02792-6_15
219

220 Z. Kakushadze and J. A. Serur
15.2 Strategy: Active Distressed Investing
This strategy amounts to buying distressed assets with the view (unlike the
passive strategy discussed above) to acquire some degree of control of the
management and direction of the company. When facing a distress situation,
a company has various options in its reorganization process. It can ﬁle for
bankruptcy protection under Chapter 11 of the U.S. Bankruptcy Code to
reorganize. Or it can work directly with its creditors out of Court. 2 Below are
some scenarios for active investing.
Strategy: Planning a Reorganization
An investor can submit a reorganization plan to Court with an objective to
obtain participation in the management of the company, attempt to increase
its value and generate proﬁts. Plans by signiﬁcant debt holders tend to be more
competitive.
Strategy: Buying Outstanding Debt
This strategy amounts to buying outstanding debt of a distressed ﬁrm at a
discount with the view that, after reorganizatsion, part of this debt will be
converted into the ﬁrm’s equity thereby giving the investor a certain level of
control of the company.
Strategy: Loan-to-own
This strategy amounts to ﬁnancing (via secured loans) a distressed ﬁrm that is
not bankrupt with the view that it (i) overcomes the distress situation, avoids
bankruptcy and increases its equity value, or (ii) ﬁles for Chapter 11 protection
and, upon reorganization, the secured loan is converted into the ﬁrm’s equity
with control rights.
15.3 Strategy: Distress Risk Puzzle
Some studies suggest that companies more prone to bankruptcy offer higher
returns, which is a form of a risk premium (see, e.g., Chan and Chen 1991;
Fama and French 1992, 1996; Vassalou and Xing 2004). However, more recent
2For some literature, see, e.g., Altman and Hotchkiss ( 2006), Chatterjee et al. ( 1996), Gilson ( 1995),
Gilson et al. ( 1990), Jostarndt and Sautner ( 2010), Levy ( 1991), Markwardt et al. ( 2016), Peri´c( 2015),
Rosenberg ( 1992), Swank and Root ( 1995), Ward and Griepentrog ( 1993).

15 Distressed Assets 221
studies suggest the opposite, i.e., that such companies do not outperform
healthier ones, and that the latter actually offer higher returns. This is the
so-called “distress risk puzzle” (see, e.g., George and Hwang 2010; Godfrey
and Brooks 2015; Grifﬁn and Lemmon 2002;O z d a g l i2010). So, this strategy
amounts to buying the safest companies and selling the riskiest ones. As a
proxy, one can use the probability of bankruptcy P
i , i = 1,..., N (N is
the number of stocks), which can, e.g., be modeled via a logistic regression
(see, e.g., Campbell et al. 2008).3 A zero-cost portfolio can be constructed
by, e.g., selling the stocks in the top decile by Pi , and buying the stocks in
the bottom decile. T ypically, the portfolio is rebalanced monthly, but annual
rebalancing is also possible (with similar returns).
Strategy: Distress Risk Puzzle—Risk Management
This strategy is a variation of the distress risk puzzle strategy in Sect. 15.3.
Empirical studies suggest that zero-cost healthy-minus-distressed (HMD)
strategies tend to have a high time-varying market beta, which turns signif-
icantly negative following market downturns (usually associated with increased
volatility), which can cause large losses if the market bounces abruptly
(see, e.g., Garlappi and Yan 2011; O’Doherty2012;O p p 2017). This is similar
to what happens in other factor-based strategies.
4 T o mitigate this, the strategy
can be modiﬁed as follows (see, e.g., Eisdorfer and Misirli 2015):
HMD∗ = σtarget
ˆσ HMD (15.1)
Here: HMD is for the standard HMD strategy in Sect. 15.3; σtarget is the level
of target volatility (typically, between 10 and 15%, depending on the trader
preferences); and ˆσ is the estimated realized volatility over the prior year using
3For some literature on models for estimating bankruptcy probabilities, explanatory variables and related
topics, see, e.g., Alaminos et al. ( 2016), Altman ( 1968, 1993), Aretz and Pope ( 2013), Beaver ( 1966),
Beaver et al. ( 2005), Bellovary et al. ( 2007), Brezigar-Masten and Masten ( 2012), Callejón et al. ( 2013),
Chaudhuri and De ( 2011), Chava and Jarrow ( 2004), Chen et al. ( 2011), Cultrera and Brédart ( 2015),
Dichev (1998), Dufﬁe et al. ( 2007), DuJardin (2015), El Kalak and Hudson ( 2016), Fedorova et al. ( 2013),
Ferreira et al. ( 2016), Gordini ( 2014), Grifﬁn and Lemmon ( 2002), Hensher and Jones ( 2007), Hillegeist
et al. ( 2004), Jo et al. ( 1997), Jonsson and Fridson ( 1996), Korol ( 2013), Laitinen and Laitinen ( 2000),
McKee and Lensberg ( 2002), Min et al. ( 2006), Mossman et al. ( 1998), Odom and Sharda ( 1990), Ohlson
(1980), Philosophov and Philosophov ( 2005), Pindado et al. ( 2008), Podobnik et al. ( 2010), Ribeiro et al.
(2012), Shin and Lee ( 2002), Shumway ( 2001), Slowinski and Zopounidis ( 1995), Tinoco and Wilson
(2013), Tsai et al. ( 2014), Wilson and Sharda ( 1994), Woodlock and Dangol ( 2014), Yang et al. ( 2011),
Zhou ( 2013), Zmijewski ( 1984).
4See, e.g., Barroso and Santa-Clara ( 2014), Blitz et al. ( 2011), Daniel and Moskowitz ( 2016).

222 Z. Kakushadze and J. A. Serur
daily data. So, 100% of the investment is allocated only if ˆσ = σtarget ,a n da
lower amount is allocated when ˆσ>σ target .W h e nˆσ<σ target , the strategy
could be leveraged. 5
References
Alaminos, D., del Castillo, A., & Fernández, M. Á. (2016). A Global Model for
Bankruptcy Prediction. PLoS ONE , 11(11), e0166693.
Altman, E. I. (1968). Financial Ratios, Discriminant Analysis and the Prediction of
Corporate Bankruptcy. Journal of Finance , 23(4), 589–609.
Altman, E. I. (1993). Corporate Financial Distress and Bankruptcy (2nd ed.). Hoboken,
NJ: Wiley.
Altman, E. I. (1998). Market Dynamics and Investment Performance of Distressed
and Defaulted Debt Securities (Working Paper). Available online: https://ssrn.com/
abstract=164502.
Altman, E. I., & Hotchkiss, E. (2006). Corporate Financial Distress and Bankruptcy:
Predict and Avoid Bankruptcy, Analyze and Invest in Distressed Debt . Hoboken, NJ:
Wiley.
Aretz, K., & Pope, P . F . (2013). Common Factors in Default Risk Across Countries
and Industries. European Financial Management , 19 (1), 108–152.
Barroso, P ., & Santa-Clara, P . (2014). Momentum Has Its Moments. Journal of Finan-
cial Economics , 116 (1), 111–120.
Beaver, W . H. (1966). Financial Ratios as Predictors of Failure. Journal of Accounting
Research, 4, 71–111.
Beaver, W . H., McNichols, M. F ., & Rhie, J.-W . (2005). Have Financial Statements
Become Less Informative? Evidence from the Ability of Financial Ratios to Predict
Bankruptcy. Review of Accounting Studies , 10 (1), 93–122.
Bellovary, J. L., Giacomino, D. E., & Akers, M. D. (2007). A Review of Bankruptcy
Prediction Studies: 1930 to Present. Journal of Financial Education , 33(4), 3–41.
Blitz, D. C., Huij, J., & Martens, M. (2011). Residual Momentum. Journal of Empir-
ical Finance , 18(3), 506–521.
Brezigar-Masten, A., & Masten, P . (2012). CART-Based Selection of Bankruptcy
Predictors for the Logit Model. Expert Systems with Applications , 39 (11), 10153–
10159.
Callejón, A. M., Casado, A. M., Fernández, M. A., & Peláez, J. I. (2013). A System
of Insolvency Prediction for Industrial Companies Using a Financial Alternative
M o d e lw i t hN e u r a lN e t w o r k s .International Journal of Computational Intelligence
Systems, 6 (1), 29–37.
Campbell, J. Y., Hilscher, J., & Sziglayi, J. (2008). In Search of Distress Risk. Journal
of Finance , 63(6), 2899–2939.
5Or, more simply, 100% of the investment could be allocated without leverage, in which case the prefactor
in Eq. ( 15.1)i sm i n (σtarget /ˆσ, 1) instead.

15 Distressed Assets 223
Chan, K. C., & Chen, N.-F . (1991). Structural and Return Characteristics of Small
and Large Firms. Journal of Finance , 46 (4), 1467–1484.
Chatterjee, S., Dhillon, U. S., & Ramírez, G. G. (1996). Resolution of Financial Dis-
tress: Debt Restructurings via Chapter 11, Prepackaged Bankruptcies, and Work-
outs. Financial Management , 25 (1), 5–18.
Chaudhuri, A., & De, K. (2011). Fuzzy Support Vector Machine for Bankruptcy
Prediction. Applied Soft Computing , 11(2), 2472–2486.
Chava, S., & Jarrow, R. A. (2004). Bankruptcy Prediction with Industry Effects.
Review of Finance , 8(4), 537–569.
Chen, H.-L., Yang, B., Wang, G., Liu, J., Xu, X., Wang, S.-J., et al. (2011). A Novel
Bankruptcy Prediction Model Based on an Adaptive Fuzzy K-nearest Neighbor
Method. Knowledge-Based Systems , 24 (8), 1348–1359.
Clark, T . A., & Weinstein, M. I. (1983). The Behavior of the Common Stock of
Bankrupt Firms. Journal of Finance , 38(2), 489–504.
Cultrera, L., & Brédart, X. (2015). Bankruptcy Prediction: The Case of Belgian SMEs.
Review of Accounting and Finance , 15 (1), 101–119.
Daniel, K., & Moskowitz, T . J. (2016). Momentum Crashes. Journal of Financial
Economics, 122(2), 221–247.
Dichev, I. (1998). Is the Risk of Bankruptcy a Systematic Risk? Journal of Finance ,
53(3), 1131–1147.
Dufﬁe, D., Saita, L., & Wang, K. (2007). Multi-Period Corporate Default Prediction
with Stochastic Covariates. Journal of Financial Economics , 83(3), 635–665.
DuJardin, P . (2015). Bankruptcy Prediction Using T erminal Failure Processes. Euro-
pean Journal of Operational Research , 242(1), 286–303.
Eberhart, A. C., Altman, E., & Aggarwal, R. (1999). The Equity Performance of Firms
Emerging from Bankruptcy. Journal of Finance , 54 (5), 1855–1868.
Eberhart, A. C., & Sweeney, R. J. (1992). Does the Bond Market Predict Bankruptcy
Settlements? Journal of Finance , 47 (3), 943–980.
Eisdorfer, A., & Misirli, E. (2015). Distressed Stocks in Distressed Times (Working
Paper). Available online: https://ssrn.com/abstract=2697771.
El Kalak, I., & Hudson, R. (2016). The Effect of Size on the Failure Probabilities
of SMEs: An Empirical Study on the US Market Using Discrete Hazard Model.
International Review of Financial Analysis , 43, 135–145.
Fama, E. F ., & French, K. R. (1992). The Cross-Section of Expected Stock Returns.
Journal of Finance , 47 (2), 427–465.
Fama, E. F ., & French, K. R. (1996). Multifactor Explanations of Asset Pricing Anoma-
lies. Journal of Finance , 51(1), 55–84.
Fedorova, E., Gilenko, E., & Dovzhenko, S. (2013). Bankruptcy Prediction for Rus-
sian Companies: Application of Combined Classiﬁers.
Expert Systems with Appli-
cations, 40 (18), 7285–7293.
Ferreira, S., Grammatikos, T ., & Michala, D. (2016). Forecasting Distress in Europe
SME Portfolios. Journal of Banking & Finance , 64, 112–135.

224 Z. Kakushadze and J. A. Serur
Friewald, N., Jankowitsch, R., & Subrahmanyam, M. (2012). Illiquidity, or Credit
Deterioration: A Study of Liquidity in the U.S. Bond Market During Financial
Crises. Journal of Financial Economics , 105 (1), 18–36.
Gande, A., Altman, E., & Saunders, A. (2010). Bank Debt vs. Bond Debt: Evidence
from Secondary Market Prices. Journal of Money, Credit and Banking , 42(4), 755–
767.
Garlappi, L., & Yan, H. (2011). Financial Distress and the Cross-Section of Equity
Returns. Journal of Finance , 66 (3), 789–822.
George, T . J., & Hwang, C.-Y. (2010). A Resolution of the Distress Risk and Leverage
Puzzles in the Cross Section of Stock Returns. Journal of Financial Economics , 96 (1),
56–79.
Gilson, S. C. (1995). Investing in Distressed Situations: A Market Survey. Financial
Analysts Journal , 51(6), 8–27.
Gilson, S. C. (2010). Creating Value Through Corporate Restructuring: Case Studies in
Bankruptcies, Buyouts, and Breakups . Hoboken, NJ: Wiley.
Gilson, S. C. (2012). Preserving Value by Restructuring Debt. Journal of Applied
Corporate Finance , 24 (4), 22–35.
Gilson, S. C., John, K., & Lang, L. H. P . (1990). T roubled Debt Restructurings: An
Empirical Study of Private Reorganization of Firms in Default. Journal of Financial
Economics, 27 (2), 315–353.
Godfrey, C., & Brooks, C. (2015). The Negative Credit Risk Premium Puzzle: A
Limits to Arbitrage Story (Working Paper). Available online: https://ssrn.com/
abstract=2661232.
Gordini, N. (2014). A Genetic Algorithm Approach for SMEs Bankruptcy Prediction:
Empirical Evidence from Italy. Expert Systems with Applications , 41(14), 6433–
6455.
Greenhaus, S. F . (1991). Approaches to Investing in Distressed Securities: Passive
Approaches. In T . A. Bowman (Ed.), Analyzing Investment Opportunities in Dis-
tressed and Bankrupt Companies (AIMR Conference Proceedings, Vol. 1991, Issue
1, pp. 47–52). Chicago, IL: AIMR.
Grifﬁn, J. M., & Lemmon, M. L. (2002). Book-to-Market Equity, Distress Risk, and
Stock Returns. Journal of Finance , 57 (5), 2317–2336.
Harner, M. M. (2008). The Corporate Governance and Public Policy Implications of
Activist Distressed Debt Investing. Fordham Law Review , 77 (2), 703–773.
Harner, M. M. (2011). Activist Distressed Debtholders: The New Barbarians at the
Gate? W ashington University Law Review , 89 (1), 155–206.
Hensher, D., & Jones, S. (2007). Forecasting Corporate Bankruptcy: Optimizing the
Performance of the Mixed Logit Model. Abacus, 43(3), 241–364.
Hillegeist, S. A., Keating, E., Cram, D. P ., & Lunstedt, K. G. (2004). Assessing the
Probability of Bankruptcy. Review of Accounting Studies , 9 (1), 5–34.
Hotchkiss, E. S., & Mooradian, R. M. (1997). Vulture Investors and the Market for
Control of Distressed Firms.
Journal of Financial Economics , 43(3), 401–432.
Jiang, W ., Li, K., & Wang, W . (2012). Hedge Funds and Chapter 11. Journal of
Finance, 67 (2), 513–560.

15 Distressed Assets 225
Jo, H., Han, I., & Lee, H. (1997). Bankruptcy Prediction Using Case-Based Reason-
ing, Neural Networks, and Discriminant Analysis. Expert Systems with Applications ,
13(2), 97–108.
Jonsson, J., & Fridson, M. (1996). Forecasting Default Rates on High Yield Bonds.
Journal of Fixed Income , 6 (1), 69–77.
Jostarndt, P ., & Sautner, Z. (2010). Out-of-Court Restructuring Versus Formal
Bankruptcy in a Non-interventionist Bankruptcy Setting. Review of Finance , 14 (4),
623–668.
Korol, T . (2013). Early Warning Models Against Bankruptcy Risk for Central Euro-
pean and Latin American Enterprises. Economic Modelling , 31, 22–30.
Laitinen, E. K., & Laitinen, T . (2000). Bankruptcy Prediction Application of the
T aylor’s Expansion in Logistic Regression.International Review of Financial Analysis ,
9 (4), 327–349.
Levy, P . S. (1991). Approaches to Investing in Distressed Securities: Active Approaches.
In T . A. Bowman (Ed.), Analyzing Investment Opportunities in Distressed and
Bankrupt Companies (AIMR Conference Proceedings, Vol. 1991, Issue 1, pp. 44–
46). Chicago, IL: AIMR.
Lhabitant, F .-S. (2002). Hedge Funds: Myths and Limits . Chichester, UK: Wiley.
Markwardt, D., Lopez, C., & DeVol, R. (2016). The Economic Impact of Chapter
11 Bankruptcy Versus Out-of-Court Restructuring. Journal of Applied Corporate
Finance, 28(4), 124–128.
McKee,T . E., & Lensberg,T . (2002). Genetic Programming and Rough Sets: A Hybrid
Approach to Bankruptcy Classiﬁcation. European Journal of Operational Research ,
138(2), 436–451.
Min, S., Lee, J., & Han, I. (2006). Hybrid Genetic Algorithms and Support Vector
Machines for Bankruptcy Prediction. Expert Systems with Applications , 31(3), 652–
660.
Morse, D., & Shaw, W . (1988). Investing in Bankrupt Firms. Journal of Finance ,
43(5), 1193–1206.
Mossman, C. E., Bell, G. G., Swartz, L. M., & T urtle, H. (1998). An Empirical
Comparison of Bankruptcy Models. Financial Review , 33(2), 35–54.
Moyer, S. G., Martin, D., & Martin, J. (2012). A Primer on Distressed Investing:
Buying Companies by Acquiring Their Debt. Journal of Applied Corporate Finance ,
24 (4), 59–76.
O’Doherty, M. S. (2012). On the Conditional Risk and Performance of Financially
Distressed Stocks. Management Science , 58(8), 1502–1520.
Odom, M. D., & Sharda, R. (1990). A Neural Network Model for Bankruptcy Pre-
diction. In Proceedings of the International Joint Conference on Neural Networks (V ol.
2, pp. 163–168). Washington, DC: IEEE.
Ohlson, J. A. (1980). Financial Ratios and the Probabilistic Prediction of Bankruptcy.
Journal of Accounting Research , 18(1), 109–131.
Opp, C. C. (2017). Learning, Optimal Default, and the Pricing of Distress Risk (Working
Paper). Available online: https://ssrn.com/abstract=2181441.

226 Z. Kakushadze and J. A. Serur
Ozdagli, A. K. (2010). The Distress Premium Puzzle (Working Paper). Available online:
https://ssrn.com/abstract=1713449.
Peri´c, M. R. (2015). Ekonomski aspekti korporativnih bankrotstava i steˇ cajnih procesa .
Belgrade, Serbia: Modern Business School.
Philosophov, L. V ., & Philosophov, V . L. (2005). Optimization of a Firm’s Capital
Structure: A Quantitative Approach Based on a Probabilistic Prognosis of Risk and
Time of Bankruptcy. International Review of Financial Analysis , 14 (2), 191–209.
Pindado, J., Rodrigues, L., & de la T orre, C. (2008). Estimating Financial Distress
Likelihood. Journal of Business Research , 61(9), 995–1003.
Podobnik, B., Horvatic, D., Petersen, A. M., Uroševi´ c, B., & Stanley, H. E. (2010).
Bankruptcy Risk Model and Empirical T ests. Proceedings of the National Academy
of Sciences , 107 (43), 18325–18330.
Putnam, G., III. (1991). Investment Opportunities in Distressed Equities. In S. Levine
(Ed.), Handbook of T urnaround and Bankruptcy Investing (pp. 196–207). New York,
NY: HarperCollins.
Quintero, R. G. (1989). Acquiring the T urnaround Candidate. In S. Levine (Ed.),
The Acquisitions Manual (pp. 379–441). New York Institute of Finance: New York,
NY.
Reiss, M. F ., & Phelps, T . G. (1991). Identifying a T roubled Company. In D. Dinapoli,
S. C. Sigoloff, & R. F . Cushman (Eds.), Workouts and T urnarounds: The Handbook of
Restructuring and Investing in Distressed Companies (pp. 7–43). Business One-Irwin:
Homewood, IL.
Ribeiro, B., Silva, C., Chen, N., Vieira, A., & das Neves, J. C. (2012). Enhanced
Default Risk Models with SVM+. Expert Systems with Applications , 39 (11), 10140–
10152.
Rosenberg, H. (1992). Vulture Investors. New York, NY: HarperCollins.
Shin, K., & Lee, Y. (2002). A Genetic Algorithm Application in Bankruptcy Prediction
Modeling. Expert Systems with Applications , 23(3), 321–328.
Shumway, T . (2001). Forecasting Bankruptcy More Accurately: A Simple Hazard
Model. Journal of Business , 74 (1), 101–104.
Slowinski, R., & Zopounidis, C. (1995). Application of the Rough Set Approach
to Evaluation of Bankruptcy Risk. Intelligent Systems in Accounting, Finance and
Management, 4 (1), 27–41.
Swank, T . A., & Root, T . H. (1995). Bonds in Default: Is Patience a Virtue? Journal
of Fixed Income , 5 (1), 26–31.
Tinoco, M. H., & Wilson, N. (2013). Financial Distress and Bankruptcy Predic-
tion Among Listed Companies Using Accounting, Market and Macroeconomic
Variables. International Review of Financial Analysis , 30, 394–419.
Tsai, C., Hsu, Y., & Yen, D. C. (2014). A Comparative Study of Classiﬁer Ensembles
for Bankruptcy Prediction. Applied Soft Computing , 24, 977–984.
Vassalou, M., & Xing, Y. (2004). Default Risk in Equity Returns. Journal of Finance ,
59 (2), 831–868.
Volpert, B. S. (1991). Opportunities for Investing in T roubled Companies. In D.
D i n a p o l i ,S .C .S i g o l o f f ,&R .F .C u s h m a n( E d s . ) ,Workouts and T urnarounds: The

15 Distressed Assets 227
Handbook of Restructuring and Investing in Distressed Companies (pp. 514–542).
Business One-Irwin: Homewood, IL.
Ward, D., & Griepentrog, G. (1993). Risk and Return in Defaulted Bonds. Financial
Analysts Journal , 49 (3), 61–65.
Wilson, R. L., & Sharda, R. (1994). Bankruptcy Prediction Using Neural Networks.
Decision Support Systems , 11(5), 545–557.
Woodlock, P ., & Dangol, R. (2014). Managing Bankruptcy and Default Risk. Journal
of Corporate Accounting & Finance , 26 (1), 33–38.
Yang, Z., You, W ., & Ji, G. (2011). Using Partial Least Squares and Support Vector
Machines for Bankruptcy Prediction. Expert Systems with Applications , 38(7), 8336–
8342.
Zhou, L. (2013). Performance of Corporate Bankruptcy Prediction Models on Imbal-
anced Dataset: The Effect of Sampling Methods. Knowledge-Based Systems , 41,
16–25.
Zmijewski, M. E. (1984). Methodological Issues Related to the Estimation of Financial
Distress Prediction Models. Journal of Accounting Research , 22, 59–82.
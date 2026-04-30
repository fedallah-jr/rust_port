# Front Matter

ZURA KAKUSHADZE
JUAN ANDRÉS SERUR
151
STRATEGIES
TRADING

151 Trading Strategies

Zura Kakushadze  Juan Andr és Serur
151 Trading Strategies

Zura Kakushadze
Quantigic Solutions LLC
Stamford, CT, USA
Juan Andrés Serur
Universidad del CEMA
Buenos Aires, Argentina
ISBN 978-3-030-02791-9 ISBN 978-3-030-02792-6 (eBook)
https://doi.org/10.1007/978-3-030-02792-6
Library of Congress Control Number: 2018958724
© The Editor(s) (if applicable) and The Author(s), under exclusive license to Springer Nature Switzerland AG,
part of Springer Nature 2018
This work is subject to copyright. All rights are solely and exclusively licensed by the Publisher, whether
the whole or part of the material is concerned, speci ﬁcally the rights of translation, reprinting, reuse
of illustrations, recitation, broadcasting, reproduction on micro ﬁlms or in any other physical way, and
transmission or information storage and retrieval, electronic adaptation, computer software, or by similar or
dissimilar methodology now known or hereafter developed.
The use of general descriptive names, registered names, trademarks, service marks, etc. in this publication does
not imply, even in the absence of a speci ﬁc statement, that such names are exempt from the relevant protective
laws and regulations and therefore free for general use.
The publisher, the authors and the editors are safe to assume that the advice and information in this book
are believed to be true and accurate at the date of publication. Neither the publisher nor the authors or the
editors give a warranty, express or implied, with respect to the material contained herein or for any errors or
omissions that may have been made. The publisher remains neutral with regard to jurisdictional claims in
published maps and institutional af ﬁliations.
Cover image: © Tetra Images/Getty Images
Cover design: Fatima Jamadar
Disclaimer: The address used by the corresponding author is for no purpose other than to indicate his profes-
sional af ﬁliation as is customary in publications. In particular, the contents of this book are not intended as an
investment, legal, tax or any other such advice, and in no way represent views of Quantigic® Solutions LLC,
the website www.quantigic.com or any of their other af ﬁliates.
This Palgrave Macmillan imprint is published by the registered company Springer Nature Switzerland AG
The registered company address is: Gewerbestrasse 11, 6330 Cham, Switzerland

ZK: To my mother Mila and my children Mirabelle and Maximilien
JAS: To my parents, Claudio and Andrea, and my brother Emiliano

Contents
1 Introduction and Summary 1
References 3
2 Options 5
2.1 Generalities 5
2.2 Strategy: Covered Call 7
2.3 Strategy: Covered Put 7
2.4 Strategy: Protective Put 8
2.5 Strategy: Protective Call 8
2.6 Strategy: Bull Call Spread 8
2.7 Strategy: Bull Put Spread 9
2.8 Strategy: Bear Call Spread 9
2.9 Strategy: Bear Put Spread 10
2.10 Strategy: Long Synthetic Forward 10
2.11 Strategy: Short Synthetic Forward 10
2.12 Strategy: Long Combo 11
2.13 Strategy: Short Combo 11
2.14 Strategy: Bull Call Ladder 12
2.15 Strategy: Bull Put Ladder 12
2.16 Strategy: Bear Call Ladder 13
2.17 Strategy: Bear Put Ladder 13
2.18 Strategy: Calendar Call Spread 14
2.19 Strategy: Calendar Put Spread 14
2.20 Strategy: Diagonal Call Spread 15
vii

2.21 Strategy: Diagonal Put Spread 16
2.22 Strategy: Long Straddle 16
2.23 Strategy: Long Strangle 17
2.24 Strategy: Long Guts 17
2.25 Strategy: Short Straddle 18
2.26 Strategy: Short Strangle 18
2.27 Strategy: Short Guts 18
2.28 Strategy: Long Call Synthetic Straddle 19
2.29 Strategy: Long Put Synthetic Straddle 19
2.30 Strategy: Short Call Synthetic Straddle 20
2.31 Strategy: Short Put Synthetic Straddle 20
2.32 Strategy: Covered Short Straddle 20
2.33 Strategy: Covered Short Strangle 21
2.34 Strategy: Strap 21
2.35 Strategy: Strip 22
2.36 Strategy: Call Ratio Backspread 22
2.37 Strategy: Put Ratio Backspread 22
2.38 Strategy: Ratio Call Spread 23
2.39 Strategy: Ratio Put Spread 23
2.40 Strategy: Long Call Butter ﬂy2 4
Strategy: Modi ﬁed Call Butter ﬂy2 4
2.41 Strategy: Long Put Butter ﬂy2 5
Strategy: Modi ﬁed Put Butter ﬂy2 5
2.42 Strategy: Short Call Butter ﬂy2 5
2.43 Strategy: Short Put Butter ﬂy2 6
2.44 Strategy: “Long” Iron Butter ﬂy2 6
2.45 Strategy: “Short” Iron Butter ﬂy2 7
2.46 Strategy: Long Call Condor 27
2.47 Strategy: Long Put Condor 28
2.48 Strategy: Short Call Condor 28
2.49 Strategy: Short Put Condor 28
2.50 Strategy: Long Iron Condor 29
2.51 Strategy: Short Iron Condor 29
2.52 Strategy: Long Box 30
2.53 Strategy: Collar 30
2.54 Strategy: Bullish Short Seagull Spread 31
2.55 Strategy: Bearish Long Seagull Spread 31
2.56 Strategy: Bearish Short Seagull Spread 32
2.57 Strategy: Bullish Long Seagull Spread 32
References 33
viii Contents

3 Stocks 41
3.1 Strategy: Price-Momentum 41
3.2 Strategy: Earnings-Momentum 43
3.3 Strategy: Value 44
3.4 Strategy: Low-Volatility Anomaly 44
3.5 Strategy: Implied Volatility 45
3.6 Strategy: Multifactor Portfolio 45
3.7 Strategy: Residual Momentum 46
3.8 Strategy: Pairs Trading 48
3.9 Strategy: Mean-Reversion — Single Cluster 49
Strategy: Mean-Reversion — Multiple Clusters 50
3.10 Mean-Reversion — Weighted Regression 52
3.11 Strategy: Single Moving Average 53
3.12 Strategy: Two Moving Averages 54
3.13 Strategy: Three Moving Averages 54
3.14 Strategy: Support and Resistance 55
3.15 Strategy: Channel 55
3.16 Strategy: Event-Driven — M&A 56
3.17 Strategy: Machine Learning — Single-Stock KNN 57
3.18 Strategy: Statistical Arbitrage — Optimization 59
Dollar-Neutrality 61
3.19 Strategy: Market-Making 63
3.20 Strategy: Alpha Combos 64
3.21 A Few Comments 66
References 67
4 Exchange-Traded Funds (ETFs) 87
4.1 Strategy: Sector Momentum Rotation 87
Strategy: Sector Momentum Rotation with MA ﬁlter 88
Strategy: Dual-Momentum Sector Rotation 88
4.2 Strategy: Alpha Rotation 89
4.3 Strategy: R-squared 89
4.4 Strategy: Mean-reversion 90
4.5 Strategy: Leveraged ETFs (LETFs) 91
4.6 Strategy: Multi-asset Trend Following 91
References 92
5 Fixed Income 99
5.1 Generalities 99
Zero-Coupon Bonds 99
Contents ix

Bonds with Coupons 99
Floating Rate Bonds 100
Swaps 101
Duration and Convexity 102
5.2 Strategy: Bullets 103
5.3 Strategy: Barbells 103
5.4 Strategy: Ladders 104
5.5 Strategy: Bond Immunization 105
5.6 Strategy: Dollar-Duration-Neutral Butter ﬂy 106
5.7 Strategy: Fifty-Fifty Butter ﬂy 107
5.8 Strategy: Regression-Weighted Butter ﬂy 107
Strategy: Maturity-Weighted Butter ﬂy 108
5.9 Strategy: Low-Risk Factor 108
5.10 Strategy: Value Factor 108
5.11 Strategy: Carry Factor 109
5.12 Strategy: Rolling Down the Yield Curve 110
5.13 Strategy: Yield Curve Spread (Flatteners and Steepeners) 110
5.14 Strategy: CDS Basis Arbitrage 111
5.15 Strategy: Swap-Spread Arbitrage 111
References 112
6 Indexes 121
6.1 Generalities 121
6.2 Strategy: Cash-and-carry Arbitrage 121
6.3 Strategy: Dispersion Trading in Equity Indexes 122
Strategy: Dispersion Trading: Subset Portfolio 123
6.4 Strategy: Intraday Arbitrage Between Index ETFs 125
6.5 Strategy: Index Volatility Targeting with Risk-Free Asset 125
References 126
7 Volatility 131
7.1 Generalities 131
7.2 Strategy: VIX Futures Basis Trading 131
7.3 Strategy: Volatility Carry with Two ETNs 133
Strategy: Hedging Short VXX with VIX Futures 133
7.4 Strategy: Volatility Risk Premium 134
Strategy: Volatility Risk Premium with Gamma Hedging 135
7.5 Strategy: Volatility Skew — Long Risk Reversal 135
7.6 Strategy: Volatility Trading with Variance Swaps 136
References 137
x Contents

8 Foreign Exchange (FX) 143
8.1 Strategy: Moving Averages with HP Filter 143
8.2 Strategy: Carry Trade 144
Strategy: High-minus-low Carry 145
8.3 Strategy: Dollar Carry Trade 146
8.4 Strategy: Momentum and Carry Combo 147
8.5 Strategy: FX Triangular Arbitrage 147
References 148
9 Commodities 155
9.1 Strategy: Roll Yields 155
9.2 Strategy: Trading Based on Hedging Pressure 156
9.3 Strategy: Portfolio Diversi ﬁcation with Commodities 156
9.4 Strategy: Value 157
9.5 Strategy: Skewness Premium 157
9.6 Strategy: Trading with Pricing Models 158
References 159
10 Futures 165
10.1 Strategy: Hedging Risk with Futures 165
Strategy: Cross-Hedging 165
Strategy: Interest Rate Risk Hedging 166
10.2 Strategy: Calendar Spread 167
10.3 Strategy: Contrarian Trading (Mean-Reversion) 168
Strategy: Contrarian Trading — Market Activity 168
10.4 Strategy: Trend Following (Momentum) 169
References 171
11 Structured Assets 181
11.1 Generalities: Collateralized Debt Obligations (CDOs) 181
11.2 Strategy: Carry, Equity Tranche — Index Hedging 183
11.3 Strategy: Carry, Senior/Mezzanine — Index Hedging 183
11.4 Strategy: Carry — Tranche Hedging 184
11.5 Strategy: Carry — CDS Hedging 184
11.6 Strategy: CDOs — Curve Trades 184
11.7 Strategy: Mortgage-Backed Security (MBS) Trading 185
References 186
12 Convertibles 193
12.1 Strategy: Convertible Arbitrage 193
Contents xi

12.2 Strategy: Convertible Option-Adjusted Spread 194
References 195
13 Tax Arbitrage 199
13.1 Strategy: Municipal Bond Tax Arbitrage 199
13.2 Strategy: Cross-Border Tax Arbitrage 199
Strategy: Cross-Border Tax Arbitrage with Options 201
References 201
14 Miscellaneous Assets 205
14.1 Strategy: In ﬂation Hedging — Inﬂation Swaps 205
14.2 Strategy: TIPS-Treasury Arbitrage 206
14.3 Strategy: Weather Risk — Demand Hedging 207
14.4 Strategy: Energy — Spark Spread 209
References 210
15 Distressed Assets 219
15.1 Strategy: Buying and Holding Distressed Debt 219
15.2 Strategy: Active Distressed Investing 220
Strategy: Planning a Reorganization 220
Strategy: Buying Outstanding Debt 220
Strategy: Loan-to-own 220
15.3 Strategy: Distress Risk Puzzle 220
Strategy: Distress Risk Puzzle — Risk Management 221
References 222
16 Real Estate 229
16.1 Generalities 229
16.2 Strategy: Mixed-Asset Diversi ﬁcation with Real Estate 230
16.3 Strategy: Intra-asset Diversi ﬁcation Within Real Estate 230
Strategy: Property Type Diversi ﬁcation 230
Strategy: Economic Diversi ﬁcation 231
Strategy: Property Type and Geographic Diversi ﬁcation 231
16.4 Strategy: Real Estate Momentum — Regional Approach 231
16.5 Strategy: In ﬂation Hedging with Real Estate 232
16.6 Strategy: Fix-and- ﬂip 232
References 233
xii Contents

17 Cash 241
17.1 Generalities 241
17.2 Strategy: Money Laundering — The Dark Side of Cash 241
17.3 Strategy: Liquidity Management 242
17.4 Strategy: Repurchase Agreement (REPO) 242
17.5 Strategy: Pawnbroking 243
17.6 Strategy: Loan Sharking 243
References 243
18 Cryptocurrencies 249
18.1 Generalities 249
18.2 Strategy: Arti ﬁcial Neural Network (ANN) 250
18.3 Strategy: Sentiment Analysis — Naïve Bayes Bernoulli 254
References 256
19 Global Macro 263
19.1 Generalities 263
19.2 Strategy: Fundamental Macro Momentum 263
19.3 Strategy: Global Macro In ﬂation Hedge 264
19.4 Strategy: Global Fixed-Income Strategy 265
19.5 Strategy: Trading on Economic Announcements 265
References 265
20 Infrastructure 269
References 270
Appendix A: R Source Code for Backtesting 275
Appendix B: Disclaimers 285
Glossary 287
Bibliography 351
Explanatory Comments for Index 459
Index 461
Contents xiii

Acronyms
ABS Asset-Backed Security
ADDV Average Daily Dollar Volume
ANN Arti ﬁcial Neural Network
ATM At-The-Money
B/P Book-to-Price
BA Banker ’s Acceptance
BICS Bloomberg Industry Classi ﬁcation System
bps Basis point
BTC Bitcoin
Btu British thermal unit
CA Commodity allocation percentage
CBOE Chicago Board Options Exchange
CD Certi ﬁcate of Deposit
CDD Cooling-Degree-Days
CDO Collateralized Debt Obligation
CDS Credit Default Swap
CFTC U.S. Commodity Futures Trading Commission
CI Core In ﬂation
CIRP Covered Interest Rate Parity
CME Chicago Mercantile Exchange
COT Commitments of Traders
CPI Consumer Price Index
CPS Cents-Per-Share
CTA Commodity Trading Advisor
DJIA Dow Jones Industrial Average
EMA Exponential Moving Average
EMSD Exponential Moving Standard Deviation
xv

ETF Exchange-Traded Fund
ETH Ethereum
ETN Exchange-Traded Note
EUR Euro
FOMC Federal Open Market Committee
FX Foreign exchange
GDP Gross Domestic Product
GICS Global Industry Classi ﬁcation Standard
HDD Heating-Degree-Days
HFT High Frequency Trading
HI Headline In ﬂation
HMD Healthy-Minus-Distressed
HML High Minus Low
HP Hedging pressure; Hodrick-Prescott
IBS Internal Bar Strength
ITM In-The-Money
JPY Japanese Yen
LETF Leveraged (inverse) ETF
LIBOR London Interbank Offer Rate
M&A Mergers and Acquisitions
MA Moving Average
ML Machine Learning
MBS Mortgage-Backed Security
MBtu 1000 Btu
MKT Market (excess) return
MMBtu 1,000,000 Btu
MOM Carhart ’s momentum factor
MSA Metropolitan Statistical Area
MTM Mark-To-Market
Mwh Megawatt hour
NYSE New York Stock Exchange
OAS Option Adjusted Spread
OTM Out-of-the-Money
P&L Pro ﬁt(s) and Loss(es)
P2P Peer-to-Peer
PCA Principal Component Analysis
REIT Real Estate Investment Trust
ReLU Recti ﬁed linear unit
REPO/repo Repurchase agreement
RMSE Root Mean Square Error
RSI Relative Strength Index
S&P Standard and Poor ’s
SIC Standard Industrial Classi ﬁcation
xvi Acronyms

SMA Simple Moving Average
SMB Small Minus Big
SGD Stochastic Gradient Descent
SS Sum of Squares
StatArb Statistical Arbitrage
STRIPS Separate Trading of Registered Interest and Principal of Securities
SUE Standardized Unexpected Earnings
SVM Support Vector Machine
TTM Time-To-Maturity
TIPS Treasury In ﬂation-Protected Securities
UIRP Uncovered Interest Rate Parity
USD U.S. Dollar
VAR Vector Autoregressive Model
VWAP Volume-Weighted Average Price
YoY Year-on-Year
Acronyms xvii

Some Math Notations
iff If and only if
max (min) Maximum (minimum)
floor ðxÞ The largest integer less than or equal x
ceilingðxÞ The smallest integer greater than or equal x
ðxÞþ maxðx; 0Þ
signðxÞ Sign of x,d e ﬁned as: þ 1 if x [0; /C0 1 if x\0; 0 if x ¼ 0
jxj Absolute value of x if x is a real number
rankðxiÞ Rank of xi when N values xi (i ¼ 1; ... ; N) are sorted in
the ascending order
expðxÞ or ex Natural exponent of x
lnðxÞ Natural log of xPN
i¼1 xi Sum of N values xi (i ¼ 1; ... ; N)
QN
i¼1 xi Product of N values xi (i ¼ 1; ... ; N)
AjB¼b (or Ajb) The value of A when some quantity B it implicitly depends
on (usually evident from the context) takes value b
f ðxÞ! min ðmaxÞ Minimizing (maximizing) f ðxÞ w.r.t. x (where x can, e.g.,
be an N-vector xi, i ¼ 1; ... ; N)
argmaxz f ðzÞ The value of z for which f ðzÞ is maximized
@f =@x The ﬁrst partial derivative of the function f (which may
depend on variables other than x) w.r.t. x
@2f =@x2 The second partial derivative of the function f (which may
depend on variables other than x) w.r.t. x
G : A 7! BG is a map from set A to set B
A /C26 B Set A is a subset of set B
xix

fij f ðiÞ¼ ag The set of values of i such that the condition f ðiÞ¼ a is
satisﬁed
minði : f ðiÞ [aÞ The minimum value of i such that the condition f ðiÞ [a
is satis ﬁed
i 2 Ji is an element of set J
jJj the number of elements of J if J is a ﬁnite set
dAB (or dA;BÞ 1i f A ¼ B; otherwise, 0 (Kronecker delta)
diagðxiÞ Diagonal N /C2 N matrix with xi (i ¼ 1; ... ; N) on its
diagonal
AT Transpose of matrix A
A/C0 1 Inverse of matrix A
EtðAÞ Expected value of A at time t
dXðtÞ An in ﬁnitesimal increment of a continuous process XðtÞ
dt An in ﬁnitesimal increment of time t
PðAjBÞ Conditional probability of A occurring assuming B is true
xx Some Math Notations
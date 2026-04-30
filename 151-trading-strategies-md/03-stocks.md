# Chapter 3: Stocks

3
Stocks
3.1 Strategy: Price-Momentum
Empirically, there appears to be certain “inertia” in stock returns known as
the momentum effect, whereby future returns are positively correlated with
past returns (see, e.g., Asness 1994; Asness et al. 2013, 2014; Grinblatt and
Moskowitz 2004; Jegadeesh and Titman 1993). Let t denote time measured
in the units of 1 month, with t = 0 corresponding to the most recent time.
Let Pi (t ) be the time series of prices (fully adjusted for splits and dividends)
for the stock labeled by i (i = 1,..., N ,w h e r eN is the number of stocks in
the trading universe). Let
Ri (t ) = Pi (t )
Pi (t + 1) − 1 (3.1)
Rcum
i = Pi (S)
Pi (S + T ) − 1 (3.2)
Rmean
i = 1
T
S+T −1∑
t =S
Ri (t )( 3.3)
Rrisk .ad j
i = Rmean
i
σi
(3.4)
σ2
i = 1
T − 1
S+T −1∑
t =S
(
Ri (t ) − Rmean
i
)2 (3.5)
© The Author(s) 2018
Z. Kakushadze and J. A. Serur, 151 T rading Strategies,
https://doi.org/10.1007/978-3-030-02792-6_3
41

42 Z. Kakushadze and J. A. Serur
Here: Ri (t ) is the monthly return; Rcum
i is the cumulative return computed
over the T-month “formation period” (usually T = 12) skipping the most
recent S-month “skip period” (usually S = 1)1; Rmean
i is the mean monthly
return computed over the formation period; Rrisk .ad j
i is the risk-adjusted mean
return over the formation period; and σi is the monthly volatility calculated
over the formation period.
The price-momentum strategy amounts to buying the best performing
stocks and selling the worst performing stocks, where the “performance” is
measured by a selection criterion based on Rcum
i , Rmean
i , Rrisk .ad j
i or some
other criterion. E.g., after the stocks are sorted by Rcum
i (in the decreasing
order), the trader can, e.g., buy stocks in the top decile (winners) and short
stocks in the bottom decile (losers).
2 This can be a zero-cost strategy, i.e., the
corresponding portfolio is dollar-neutral. Alternatively, a long-only portfolio
can be constructed by buying stocks in, e.g., the top decile. Once a portfo-
lio is established at t = 0, it is kept unaltered during a predeﬁned “holding
period”,3 which can be 1 month or longer (longer holding period portfolios
typically have diminishing returns before trading costs as the momentum effect
fades with time). Multi-month-holding portfolios can be constructed by over-
lapping 1-month-holding portfolios (see, e.g., Jegadeesh and Titman 1993).
The above prescription does not ﬁx the relative weights wi of the stocks in
the portfolio. For a long-only portfolio we have wi ≥ 0 and
N∑
i =1
wi = 1 (3.6)
So, if the total investment level is I , then the stock labeled by i has I ×wi
dollars invested in it. This, up to rounding, translates into Qi = I ×wi /Pi (0)
shares.4 One can simply take uniform weights, wi = 1/N for all stocks,
albeit other weighting schemes are possible. E.g., we can have nonuniform
wi ∝ 1/σi ,o r wi ∝ 1/σ2
i ,e t c .
1Usually, the most recent month is skipped due to an empirically observed mean-reversion (a.k.a. contrar-
ian) effect in monthly returns possibly rooted in liquidity/microstructure issues—see, e.g., Asness ( 1994),
Boudoukh et al. ( 1994), Grinblatt and Moskowitz ( 2004), Jegadeesh ( 1990), Lo and MacKinlay ( 1990).
2There is some degree of arbitrariness in deﬁning winners and losers.
3Albeit, e.g., a long-only portfolio may have to be liquidated before the end of this holding period due to
unforeseen events, such as market crashes.
4That is, assuming the stock is bought at the price Pi (0), which does not account for slippage.

3 Stocks 43
For a dollar-neutral portfolio we can have negative wi and
N∑
i =1
|wi |= 1 (3.7)
N∑
i =1
wi = 0 (3.8)
So, if the total investment level is I = IL + IS ,w h e r e IL is the total long
investment, and IS is the absolute value of the total short investment, 5 then
the stock labeled by i has I × wi dollars invested in it, where wi > 0 for long
stocks, and wi < 0 for short stocks. One can simply take modulus-uniform
weights, where wi = 1/2 NL for all NL long stocks, and wi =− 1/2 NS for
all NS short stocks. However, other weighting schemes are possible, e.g., as
above, weights suppressed by σi , σ2
i ,e t c .6
3.2 Strategy: Earnings-Momentum
This strategy amounts to buying winners and selling losers as in the price-
momentum strategy, but the selection criterion is based on earnings. One way
to deﬁne such a selection criterion is via standardized unexpected earnings
(SUE) (Chan et al. 1996)
7:
SUEi = Ei − E ′
i
σi
(3.9)
Here: Ei is the most recently announced quarterly earnings per share of the
stock labeled by i ; E ′
i is the earnings per share announced 4 quarters ago;
5For dollar-neutral portfolios IL = IS and I = 2 × IL .
6For some additional literature on momentum strategies, see, e.g., Antonacci ( 2017), Asem and Tian
(2010), Barroso and Santa-Clara ( 2014), Bhojraj and Swaminathan ( 2006), Chordia and Shivakumar
(2002), Chuang and Ho ( 2014), Cooper et al. ( 2004), Daniel and Moskowitz ( 2016), Géczy and Samonov
(2016), Grifﬁn et al. ( 2003), Grundy and Martin ( 2001), Hwang and George ( 2004), Jegadeesh and
Titman (2001), Karolyi and Kho ( 2004), Korajczyk and Sadka ( 2004), Liu and Zhang ( 2008), Moskowitz
and Grinblatt ( 1999), Rouwenhorst ( 1998), Sadka ( 2002), Siganos and Chelley-Steeley ( 2006), Stivers
and Sun ( 2010).
7Also see, e.g., Bartov et al. ( 2005), Battalio and Mendenhall ( 2007), Bernard and Thomas ( 1989), Bernard
and Thomas (1990), Bhushan ( 1994), Chordia et al. ( 2009), Chordia and Shivakumar ( 2006), Czaja et al.
(2013), Doyle et al. ( 2006), Foster et al. ( 1984), Hew et al. ( 1996), Hirshleifer et al. ( 2009), Jansen and
Nikiforov ( 2016), Livnat and Mendenhall ( 2006), Loh and Warachka ( 2012), Mendenhall ( 2004), Ng
et al. ( 2008), Rendleman et al. ( 1982), Stickel ( 1991), Watts ( 1978).

44 Z. Kakushadze and J. A. Serur
σi is the standard deviation of the unexpected earnings Ei − E ′
i over the
last 8 quarters. Similarly to the price-momentum strategy, the trader can,
e.g., construct a dollar-neutral portfolio by buying stocks in the top decile by
SUE, and shorting stocks in the bottom decile. 8
3.3 Strategy: Value
This strategy amounts to buying winners and selling losers as in the price-
momentum and earnings-momentum strategies, but the selection criterion is
based on value. Value can be deﬁned as the Book-to-Price (B/P) ratio (see, e.g.,
Rosenberg et al. 1985). Here “Book” is the company’s book value per share
outstanding (so the B/P ratio is the same as the Book-to-Market ratio, where
now “Book” stands for its total book value, not per share outstanding, and
“Market” is its market capitalization). The trader can, e.g., construct a zero-
cost portfolio by buying stocks in the top decile by the B/P ratio, and shorting
stocks in the bottom decile. There can be variations in the deﬁnition of the B/P
ratio. Thus, e.g., Asness et al. ( 2013) uses current (i.e., most up-to-date) prices,
while Fama and French ( 1992) and some others use prices contemporaneous
with the book value.
9
3.4 Strategy: Low-Volatility Anomaly
This strategy is based on the empirical observation that future returns of pre-
viously low-return-volatility portfolios outperform those of previously high-
return-volatility portfolios, 10 which goes counter to the “naïve” expectation
that higher risk assets should yield proportionately higher returns. Thus, if σi
is deﬁned as the historical volatility (computed over a time series of histor-
ical returns, as in Eq. ( 3.5)), the trader can, e.g., construct a dollar-neutral
portfolio by buying stocks in the bottom decile by σi (low-volatility stocks),
and shorting stocks in the top decile (high-volatility stocks). The length of
the sample used for computing the historical volatility can, e.g., be 6 months
8T ypically, the holding period is 6 months, with diminishing returns for longer holding periods.
9The holding period typically is 1–6 months. For some additional literature on value strategies, see, e.g.,
Erb and Harvey ( 2006), Fama and French ( 1993, 1996, 1998, 2012), Fisher et al. ( 2016), Gerakos and
Linnainmaa ( 2012), Novy-Marx ( 2013), Piotroski ( 2000), Piotroski and So ( 2012), Stattman ( 1980),
Suhonen et al. ( 2017), Zhang ( 2005).
10See, e.g., Ang et al. ( 2006, 2009), Baker et al. ( 2011), Black ( 1972), Blitz and van Vliet ( 2007), Clarke
et al. ( 2006, 2010), Frazzini and Pedersen ( 2014), Fu ( 2009), Garcia-Feijóo et al. ( 2015), Li et al. ( 2014,
2016), Merton ( 1987).

3 Stocks 45
(126 trading days) to a year (252 trading days), with a similar duration for the
holding period (with no “skip period” required).
3.5 Strategy: Implied Volatility
This strategy is based on the empirical observation that stocks with larger
increases in call implied volatilities over the previous month on average
have higher future returns, while stocks with larger increases in put implied
volatilities over the previous month on average have lower future returns
(see, e.g., An et al. 2014;C h e ne ta l . 2016).11 Therefore, the trader can,
e.g., construct a dollar-neutral portfolio by buying stocks in the top decile by
the increase in call implied volatilities, and shorting stocks in the top decile
by the increase in put implied volatilities. One can also consider variations,
e.g., buying stocks in the top decile by the difference twixt the change in call
implied volatilities and the change in put implied volatilities.
3.6 Strategy: Multifactor Portfolio
This strategy amounts to buying and shorting stocks based on multiple factors
such as value, momentum, etc. For instance, usually value and momentum are
negatively correlated and combining them can add value (see, e.g., Asness et al.
2013) .T h e r ei sav a r i e t yo fw a y si nw h i c hF > 1 factors can be combined. 12
The simplest way is to diversify the exposure to the F factors with some weights
wA,w h e r eA = 1,..., F labels the factors. That is, if I is the total investment
level, then the F portfolios (each built as above based on the corresponding
factor) are allocated the investment levels IA = wA × I , where (assuming all
wA > 0)
F∑
A=1
wA = 1 (3.10)
Thus, one can simply take uniform weights wA = 1/F , albeit this may not
be the most optimal weighting scheme. E.g., similarly to Sect. 3.1,t h e r ea r e
weighting schemes with wA ∝ 1/σA, wA ∝ 1/σ2
A, etc., where σA is the
11Also see, e.g., Bali and Hovakimian ( 2009), Bollen and Whaley ( 2004), Busch et al. ( 2011), Chakravarty
et al. ( 2004), Conrad et al. ( 2013), Cremers and Weinbaum ( 2010), Pan and Poteshman ( 2006), Xing
et al. ( 2010).
12And the holding period depends on which factors are combined.

46 Z. Kakushadze and J. A. Serur
historical volatility for the corresponding factor portfolio (uniformly normal-
ized, e.g., per dollar invested). 13
Alternatively, consider F rankings of stocks based on the F factors. One can
now combine these rankings in various ways to blend the factors. E.g., in the
case of two factors, momentum and value, one can take the top (winners) and
bottom (losers) quintiles by momentum and further split them into top half
and bottom half, respectively, by value. Or one can take the top and bottom
quintiles by value and split them by momentum.
14 Yet another way is to deﬁne
demeaned ranks
sAi = rank( f Ai ) − 1
N
N∑
j =1
rank( f Aj )( 3.11)
where f Ai is the numeric value of the factor labeled by A (e.g., momentum)
for the stock labeled by i (i = 1,..., N ). One can then simply average the
ranks:
si = 1
F
F∑
A=1
sAi (3.12)
The combined “score” si can have ties, which, if need be (e.g., if there is an
ambiguity at the border of the top decile) can be resolved, e.g., simply by
giving preference to one of the factor rankings. Averaging over sAi simply
minimizes the sum of squares of the Euclidean distances between the N -vector
si and the KN -vectors sAi . One can introduce nonuniform weights into this
sum (which would amount to a weighted average in Eq. ( 3.12)), or even use
a different deﬁnition of the distance (e.g., the Manhattan distance), which
would complicate the problem computationally. Etc. 15
3.7 Strategy: Residual Momentum
This is the same as the price-momentum strategy with the stock returns Ri (t )
replaced by the residuals ϵi (t ) of a serial regression of the stock returns Ri (t )
13Another approach is to ﬁx the weights wA by optimizing a portfolio of the F expected returns corre-
sponding to the F factors (using an invertible F × F covariance matrix for these returns).
14These two ways generally do not produce the same resultant portfolios.
15For additional literature on multifactor strategies, see, e.g., Amenc et al. ( 2015, 2016), Arnott et al.
(2013), Asness ( 1997), Barber et al. ( 2015), Cochrane ( 1999), Fama ( 1996), Grinold and Kahn ( 2000),
Hsu et al. ( 2018), Kahn and Lemmon ( 2015, 2016), Kozlov and Petajisto ( 2013), Malkiel ( 2014), Wang
(2005).

3 Stocks 47
over, e.g., the 3 Fama-French factors MKT (t ),S M B(t ), HML (t ),16 with the
intercept (see, e.g., Blitz et al. 2011)17:
Ri (t ) = αi + β1,i MKT(t ) + β2,i SMB(t ) + β3,i HML(t ) + ϵi (t )( 3.13)
The regression is run over a 36-month period (Blitz et al. 2011) (with the 1-
month skip period) to estimate the regression coefﬁcients αi , β1,i , β2,i , β3,i .
Once the coefﬁcients are estimated, the residuals can be computed for the
12-month formation period (again, with the 1-month skip period):
ϵi (t ) = Ri (t ) − β1,i MKT(t ) − β2,i SMB(t ) − β3,i HML(t )( 3.14)
Note that αi is not included in this computation of the residuals for the 12-
month formation period as αi was computed for the 36-month period. These
residuals ϵi (t ) are then used to compute, e.g., the risk-adjusted residual returns
˜Rrisk .ad j
i (here S = 1 and T = 12; the holding period typically is 1 month,
but can be longer):
ϵmean
i = 1
T
S+T −1∑
t =S
ϵi (t )( 3.15)
˜Rrisk .ad j
i = ϵmean
i
˜σi
(3.16)
˜σ2
i = 1
T − 1
S+T −1∑
t =S
(
ϵi (t ) − ϵmean
i
)2 (3.17)
E.g., a dollar-neutral portfolio can be constructed by buying stocks in the
top decile by ˜Rrisk .ad j
i , and shorting stocks in the bottom decile (with
(non)uniform weights).
16The stock returns Ri are deﬁned in excess of the risk-free rate (the one-month T reasury bill rate); MKT
is the excess return of the market portfolio; SMB is the excess return of the Small minus Big (by market
capitalization) portfolio; HML is the excess return of the High minus Low (by book-to-market) portfolio.
See, e.g., Carhart ( 1997), Fama and French ( 1993) for details.
17For some additional literature related to the residual momentum strategy, see, e.g., Blitz et al. ( 2013),
Chang et al. ( 2016), Chaves ( 2012), Chuang ( 2015), Grundy and Martin ( 2001), Gutierrez and Prinsky
(2007), Hühn and Scholz ( 2017), Huij and Lansdorp ( 2017), Van Oord ( 2016).

48 Z. Kakushadze and J. A. Serur
3.8 Strategy: Pairs Trading
This dollar-neutral strategy amounts to identifying a pair of historically highly
correlated stocks (call them stock A and stock B) and, when a mispricing (i.e., a
deviation from the high historical correlation) occurs, shorting the “rich” stock
and buying the “cheap” stock. This is an example of a mean-reversion strategy.
Let PA(t1) and PB (t1) be the prices of stock A and stock B at time t1,a n d
let PA(t2) and PB (t2) be the prices of stock A and stock B at a later time t2.
All prices are fully adjusted for any splits and dividends. The corresponding
returns (from t1 to t2)a r e
RA = PA(t2)
PA(t1) − 1 (3.18)
RB = PB (t2)
PB (t1) − 1 (3.19)
Since typically these returns are small, we can use an alternative deﬁnition:
RA = ln
( PA(t2)
PA(t1)
)
(3.20)
RB = ln
( PB (t2)
PB (t1)
)
(3.21)
Next, let ˜RA and ˜RB be the demeaned returns:
R = 1
2 (RA + RB ) (3.22)
˜RA = RA − R (3.23)
˜RB = RB − R (3.24)
where R is the mean return. A stock is “rich” if its demeaned return is positive,
and it is “cheap” if its demeaned return is negative. The numbers of shares Q A,
Q B to short/buy are ﬁxed by the total desired dollar investment I (Eq. 3.25)
and the requirement of dollar-neutrality (Eq. 3.26):
PA |Q A| + PB |Q B | = I (3.25)
PA Q A + PB Q B = 0 (3.26)

3 Stocks 49
where PA, PB are the stock prices at the time t∗ the position is established
(t∗ ≥ t2).18
3.9 Strategy: Mean-Reversion—Single Cluster
This is a generalization of the pairs trading strategy to N > 2 stocks that are
historically highly correlated (e.g., stocks belonging to the same industry or
sector). Let Ri , i = 1,..., N , be the returns for these N stocks:
Ri = ln
( Pi (t2)
Pi (t1)
)
(3.27)
R = 1
N
N∑
i =1
Ri (3.28)
˜Ri = Ri − R (3.29)
Following the pairs trading intuition, we can short stocks with positive ˜Ri and
buy stocks with negative ˜Ri . We have the following conditions:
N∑
i =1
Pi |Qi | = I (3.30)
N∑
i =1
Pi Qi = 0 (3.31)
Here: I is the total desired dollar investment; Eq. ( 3.31) is the dollar-neutrality
constraint; Qi < 0 for short-sales; Qi > 0 for buys; Pi are the prices at the
time the position is established. We have 2 equations and N > 2 unknowns.
A simple prescription (which is one out of myriad possibilities) for specifying
Qi is to have the dollar positions Di = Pi Qi proportional to the demeaned
returns:
Di =− γ ˜Ri (3.32)
18For some literature on pairs trading, see, e.g., Bogomolov ( 2013), Bowen and Hutchinson ( 2016),
Bowen et al. ( 2010), Caldeira and Moura ( 2013), Chen et al. ( 2017), Do and Faff ( 2010, 2012), Elliott
et al. ( 2005), Engle and Granger ( 1987), Gatev et al. ( 2006), Huck ( 2009, 2015), Huck and Afawubo
(2014), Jacobs and Weber ( 2015), Kakushadze ( 2015), Kim ( 2011), Kishore ( 2012), Krauss ( 2017),
Krauss and Stübinger ( 2017), Liew and Wu ( 2013), Lin et al. ( 2006), Liu et al. ( 2017), Miao ( 2014),
Perlin (2009), Pizzutilo ( 2013), Rad et al. ( 2016), Stübinger and Bredthauer ( 2017), Stübinger and Endres
(2017), Vaitonis and Masteika ( 2016), Vidyamurthy ( 2004), Xie et al. ( 2014), Yoshikawa ( 2017), Zeng
and Lee ( 2014).

50 Z. Kakushadze and J. A. Serur
where γ> 0 (recall that we short ˜Ri > 0 stocks and buy ˜Ri < 0 stocks).
Then Eq. ( 3.31) is automatically satisﬁed, while Eq. ( 3.30)ﬁ x e s γ:
γ = I
∑ N
i =1
⏐⏐˜Ri
⏐⏐ (3.33)
Strategy: Mean-Reversion—Multiple Clusters
The mean-reversion strategy of Sect. 3.9 can be readily generalized to the case
where we have K > 1 clusters such that stocks within each cluster are histori-
cally highly correlated. 19 We can simply treat clusters independently from each
other and construct a mean-reversion strategy following the above procedure
in each cluster. Then, e.g., we can allocate investments to these K independent
strategies uniformly.
There is a neat way of treating all clusters in a “uniﬁed” fashion using a
linear regression. Let the K clusters be labeled by A = 1,..., K .L e t /Lambda1iA be
an N × K matrix such that if the stock labeled by i (i = 1,..., N ) belongs to
the cluster labeled by A,t h e n /Lambda1iA = 1; otherwise, /Lambda1iA = 0. We will assume
that each and every stock belongs to one and only one cluster (so there are no
empty clusters):
N A =
N∑
i =1
/Lambda1iA > 0 (3.34)
N =
K∑
A=1
N A (3.35)
We have
/Lambda1iA = δG(i ),A (3.36)
G :{ 1,..., N } ↦→{ 1,..., K } (3.37)
Here: G is the map between stocks and clusters; and /Lambda1iA is the loadings matrix.
19E.g., these clusters can correspond to sectors, such as energy, technology, healthcare, etc.

3 Stocks 51
Now consider a linear regression of the stock returns Ri over /Lambda1iA (without
the intercept and with unit weights):
Ri =
K∑
A=1
/Lambda1iA f A + εi (3.38)
where f A are the regression coefﬁcients given by (in matrix notation, where R
is the N -vector Ri , f is the K -vector f A,a n d /Lambda1is the N × K matrix /Lambda1iA )
f = Q−1 /Lambda1T R (3.39)
Q = /Lambda1T /Lambda1( 3.40)
and εi are the regression residuals. For binary /Lambda1iA given by Eq. ( 3.36), these
residuals are nothing but the returns Ri demeaned w.r.t. to the corresponding
cluster:
ε = R − /Lambda1Q−1 /Lambda1T R (3.41)
Q AB = N A δAB (3.42)
R A = 1
N A
∑
j ∈JA
R j (3.43)
εi = Ri − RG(i ) = ˜Ri (3.44)
where R A is the mean return for the cluster labeled by A,a n d ˜Ri is the
demeaned return obtained by subtracting from Ri the mean return for the
cluster labeled by A = G(i ) to which the stock labeled by i belongs:
JA ={ i |G(i ) = A}⊂{ 1,..., N }.
The demeaned returns are cluster-neutral, i.e.,
N∑
i =1
˜Ri /Lambda1iA = 0, A = 1,..., K (3.45)
Also, note that we automatically have (so Di given by Eq. ( 3.32) satisfy
Eq. ( 3.31))
N∑
i =1
˜Ri νi = 0 (3.46)

52 Z. Kakushadze and J. A. Serur
where νi ≡ 1, i = 1,..., N , i.e., the N -vector ν is the unit vector. In the
regression language, ν is the intercept. We did not have to add the intercept
to the loadings matrix /Lambda1as it is already subsumed in it:
K∑
A=1
/Lambda1iA = νi (3.47)
3.10 Mean-Reversion—Weighted Regression
The conditions ( 3.45) satisﬁed by the demeaned returns when the loadings
matrix is binary simply mean that these returns are cluster-neutral, i.e., orthog-
onal to the KN -vectors v
(A) comprising the columns of /Lambda1iA . Such orthog-
onality can be deﬁned for any loadings matrix, not just a binary one. So, we
can consider a generalization where the loadings matrix, call it /Omega1iA ,m a yh a v e
some binary columns, but generally it need not. The binary columns, if any,
can, e.g., be industry (or sector) based risk factors; the non-binary columns
are interpreted as some non-industry based risk factors; and the orthogonality
condition
N∑
i =1
˜Ri /Omega1iA , A = 1,..., K (3.48)
can be satisﬁed if the twiddled returns ˜Ri are related to the residuals εi of
the regression of Ri over /Omega1iA with some (generally nonuniform) regression
weights zi via
˜R = Z ε( 3.49)
ε = R − /Omega1Q−1 /Omega1T ZR (3.50)
Z = diag(zi )( 3.51)
Q = /Omega1T Z /Omega1( 3.52)
If the intercept is included in /Omega1iA (i.e., a linear combination of the columns
of /Omega1iA equals the unit N -vector ν), then we automatically have
N∑
i =1
˜Ri = 0 (3.53)

3 Stocks 53
The weights zi can, e.g., be taken as zi = 1/σ2
i ,w h e r e σi are historical
volatilities.20
3.11 Strategy: Single Moving Average
This strategy is based on the stock price crossing a moving average. One can
use different types of moving averages (MAs), such as a simple moving average
(SMA), or an exponential moving average (EMA) 21:
SMA(T ) = 1
T
T∑
t =1
P(t )( 3.54)
EMA(T ,λ )=
∑ T
t =1 λt −1 P(t )
∑ T
t =1 λt −1 = 1 − λ
1 − λT
T∑
t =1
λt −1 P(t )( 3.55)
Here: t = 1 corresponds to the most recent time in the time series of historical
stock prices P(t ); T is the length of the MA ( t and T are usually measured
in trading days); and λ< 1 is the factor which suppresses past contributions.
Below MA will refer to SMA or EMA. A simple strategy is deﬁned as follows
(P is the price at t = 0, on the trading day immediately following the most
recent trading day t = 1 in the time series P(t )):
Signal =
{
Establish long/liquidate short position if P > MA(T )
Establish short/liquidate long position if P < MA(T ) (3.56)
This strategy can be run as, e.g., long-only, short-only, or both long and short.
It can be straightforwardly applied to multiple stocks (on a single-stock basis,
20For some literature on mean-reversion (a.k.a. contrarian) strategies, see, e.g., Avellaneda and Lee ( 2010),
Black and Litterman ( 1991, 1992), Cheung ( 2010), Chin et al. ( 2002), Conrad and Kaul ( 1998), Daniel
(2001), Da Silva et al. ( 2009), Doan et al. ( 2014), Drobetz ( 2001), Hodges and Carverhill ( 1993),
Idzorek ( 2007), Jansen and Nikiforov ( 2016), Jegadeesh and Titman ( 1995), Kakushadze ( 2015), Kang
et al. ( 2002), Kudryavtsev ( 2012), Lakonishok et al. ( 1994), Lehmann ( 1990), Li et al. ( 2012), Liew and
Roberts ( 2013), Lo and MacKinlay ( 1990), Mun et al. ( 2000), O’T ool (2013), Pole ( 2007), Poterba and
Summers (1988), Satchell and Scowcroft ( 2000), Schiereck et al. ( 1999), Shi et al. ( 2015), Yao ( 2012).
21For T ≫ 1 we have λT ≪ 1 and EMA (T ,λ ) ≈ (1 − λ) P(1) + λ EMA(T − 1,λ ),w h e r e
EMA(T − 1,λ )is based on P(2), P(3) ,..., P(T ). Also, for some literature on moving average based
strategies, see, e.g., BenZion et al. ( 2003), Brock et al. ( 1992), Dzikeviˇcius and Šaranda ( 2010), Edwards
and Magee ( 1992), Faber ( 2007), Félix and Rodríguez ( 2008), Fiﬁeld et al. ( 2008), Fong and Yong
(2005), Gençay ( 1996, 1998), Gençay and Stengos ( 1998), Glabadanidis ( 2015), Gunasekarage and
Power ( 2001), Hung ( 2016), James ( 1968), Jasemi and Kimiagari ( 2012), Kilgallen ( 2012), Li et al.
(2015), Lo et al. ( 2000), Metghalchi et al. ( 2012), Pätäri and Vilska ( 2014), T aylor and Allen ( 1992),
Weller et al. ( 2009), Zakamulin ( 2014, 2015).

54 Z. Kakushadze and J. A. Serur
with no cross-sectional interaction between the signals for individual stocks).
With a large number of stocks, it may be possible to construct (near-)dollar-
neutral portfolios.
3.12 Strategy: T wo Moving Averages
The simplest variant of this strategy replaces the stock price P in Eq. ( 3.56)b y
another moving average. That is, we have 2 moving averages with lengths T ′
and T ,w h e r eT ′ < T (e.g., T ′ = 10 and T = 30), and the signal is given by:
Signal =
{
Establish long/liquidate short position if MA (T ′)> MA(T )
Establish short/liquidate long position if MA (T ′)< MA(T ) (3.57)
This signal can be augmented with additional “stop-loss” rules to protect real-
ized proﬁts. E.g., if a long position has been established, the trader can deﬁne
a threshold to liquidate the long position if the stock begins to fall (even if the
shorter moving average has not crossed the longer moving average yet):
Signal =
⎧
⎪⎪
⎪
⎨
⎪⎪
⎪
⎩
Establish long position if MA (T
′)> MA(T )
Liquidate long position if P <( 1 − /Delta1)× P1
Establish short position if MA (T ′)< MA(T )
Liquidate short position if P >( 1 + /Delta1)× P1
(3.58)
Here /Delta1is some predeﬁned percentage, e.g., /Delta1= 2%. So, a long position is
liquidated if the current price P falls over 2% below the previous day’s price
P1; and a short position is liquidated if P rises over 2% above P1.O t h e r
variations can be used.
3.13 Strategy: Three Moving Averages
In some cases, using 3 moving averages with lengths T1 < T2 < T3
(e.g., T1 = 3, T2 = 10, T3 = 21) can help ﬁlter false signals:
Signal =
⎧
⎪⎪⎪
⎨
⎪⎪⎪
⎩
Establish long position if MA (T
1)> MA(T2)> MA(T3)
Liquidate long position if MA (T1) ≤ MA(T2)
Establish short position if MA (T1)< MA(T2)< MA(T3)
Liquidate short position if MA (T1) ≥ MA(T2)
(3.59)

3 Stocks 55
3.14 Strategy: Support and Resistance
This strategy uses “support” S and “resistance”R levels, which can be computed
using the “pivot point” (a.k.a. the “center”) C as follows 22:
C = PH + PL + PC
3 (3.60)
R = 2 × C − PL (3.61)
S = 2 × C − PH (3.62)
Here PH , PL and PC are the previous day’s high, low and closing prices. One
way to deﬁne a trading signal is as follows (as above, P is the current price):
Signal =
⎧
⎪⎪
⎪⎨
⎪⎪
⎪
⎩
Establish long position if P > C
Liquidate long position if P ≥ R
Establish short position if P < C
Liquidate short position if P ≤ S
(3.63)
3.15 Strategy: Channel
This strategy amounts to buying and selling a stock when it reaches the ﬂoor
and the ceiling of a channel, respectively. A channel is a range/band, bounded
by a ceiling and a ﬂoor, within which the stock price ﬂuctuates. The trader’s
expectation may be that if the ﬂoor or the ceiling is reached, the stock price will
bounce in the opposite direction. On the other hand, if the stock price breaks
through the ceiling or the ﬂoor, the trader may conclude that a new trend has
emerged and follow this new trend instead. A simple and common deﬁnition
of a channel is the Donchian Channel (Donchian 1960), where the ceiling
22Other deﬁnitions of the pivot point (e.g., using the current trading day’s open price) and higher/lower
support/resistance levels exist. For some literature on support and resistance strategies, see, e.g., Amiri
et al. ( 2010), Brock et al. ( 1992), Garzarelli et al. ( 2014), Hardy ( 1978), Kahneman and Tversky ( 1979),
Murphy (1986), Osler ( 2000, 2003), Person (2007), Pring ( 1985), Shiu and Lu ( 2011), Thomsett (2003),
Zapranis and Tsinaslanidis ( 2012).

56 Z. Kakushadze and J. A. Serur
Bup and the ﬂoor Bdown are deﬁned as follows (with the same notations as
above)23:
Bup = max(P(1), P(2) ,..., P(T )) ( 3.64)
Bdown = min(P(1), P(2) ,..., P(T )) ( 3.65)
A simple trading strategy then is as follows:
Signal =
{
Establish long/liquidate short position if P = Bdown
Establish short/liquidate long position if P = Bup
(3.66)
The wider the channel, the higher the volatility. Usually, the channel indicator
is used together with other indicators. E.g., the signal can be more robust
when a price reversal (or a channel break) occurs with an increase in the traded
volume.
3.16 Strategy: Event-Driven—M&A
This strategy, referred to as “merger arbitrage” or “risk arbitrage”, attempts
to capture excess returns generated via corporate actions such as mergers and
acquisitions (M&A). A merger arbitrage opportunity arises when one publicly
traded company intends to acquire another publicly traded company at a price
that differs from the latter’s market price. In this regard, there are two main
types of transactions: cash mergers and stock mergers. In the case of a cash
merger, the trader establishes a long position in the target company stock. In
the case of a stock merger, the trader establishes a long position in the target
company stock (call it A) and a short position in the acquirer company stock
(call it B). For instance, if the current price of A is $67, the current price of B
is $35, and under the proposed stock merger deal each share of A is swapped
for 2 shares of B, then the trader buys one share of A and shorts 2 shares of B
generating an initial net credit of $3 = 2 ×$35 −$67,w h i c hi st h ep r o ﬁ tp e r
each share of A bought if the deal goes through. The trader’s risk is in that, if
the deal falls through, the trader will likely lose money on this trade.
24
23For some additional literature on channel trading strategies, see, e.g., Batten and Ellis ( 1996), Birari and
Rode ( 2014), Dempster and Jones ( 2002), De Zwart et al. ( 2009), Elder ( 2014), Sullivan et al. ( 1999).
24For some literature on merger arbitrage, see, e.g., Andrade et al. ( 2001), Andrie¸s and Vîrlan ( 2017),
Baker et al. ( 2012), Baker and Sava¸soglu ( 2002), Bester et al. ( 2017), Brown and Raymond ( 1986), Cao
et al. ( 2016), Cornelli and Li ( 2002), Dukes et al. ( 1992), Hall et al. ( 2013), Harford ( 2005), Hsieh and
Walkling (2005), Huston ( 2000), Jetley and Ji ( 2010), Karolyi and Shannon ( 1999), Khan ( 2002), Larker

3 Stocks 57
3.17 Strategy: Machine Learning—Single-Stock
KNN
Some strategies rely on machine learning techniques, such as the k-nearest
neighbor (KNN) algorithm (see, e.g., Altman 1992;S a m w o r t h2012), to pre-
dict future stock returns (the target variable) based on a set of predictor (feature)
variables, which can be based on technical, fundamental and/or some other
data. The strategy we describe here is a single-stock strategy, i.e., for each stock
the target variable is predicted using the price and volume data only for this
stock (but no cross-sectional data, i.e., no data for other stocks). The target
variable Y (t ) is deﬁned as the cumulative return over the next T trading days
(as above, the ascending integer values of t , which is measured in trading days,
correspond to going back in time):
Y (t ) = P(t − T )
P(t ) − 1 (3.67)
The predictor variables Xa (t ), a = 1,..., m, are deﬁned using prices P(t ′)
and volumes V (t ′) at times t ′ before t (i.e., t ′ > t ), so they are out-of-sample.
Examples of such variables are moving averages of the price and volume of
varying lengths:
X
1(t ) = 1
T1
T1∑
s=1
V (t + s)( 3.68)
X2(t ) = 1
T2
T2∑
s=1
P(t + s)( 3.69)
X3(t ) = 1
T3
T3∑
s=1
P(t + s)( 3.70)
... ( 3.71)
The predictor variables are further normalized to lie between 0 and 1:
˜Xa (t ) = Xa (t ) − X −
a
X +a − X −a
(3.72)
and Lys ( 1987), Lin et al. ( 2013), Maheswaran and Yeoh ( 2005), Mitchell and Pulvino ( 2001), Ofﬁcer
et al. ( 2004, 2006), Samuelson and Rosenthal ( 1986), Subramanian ( 2004), Van T assel (2016), Walkling
(1985).

58 Z. Kakushadze and J. A. Serur
where X +
a and X −
a are the maximum and minimum values of Xa (t ) over the
training period. The ﬁnal ingredient is the number k of the nearest neighbors
(see below). For a given value of t we can take k nearest neighbors of the
m-vector ˜Xa (t ) among the m-vectors ˜Xa (t ′), t ′ = t + 1,t + 2,..., t + T∗,
using the KNN algorithm (here T∗ is the sample size). For KNN we can use
the Euclidean distance D(t,t ′) between ˜Xa (t ) and ˜Xa (t ′) deﬁned as
[D(t,t ′)]2 =
m∑
a=1
(˜Xa (t ) − ˜Xa (t ′))2 (3.73)
However, we can use some other distance (e.g., the Manhattan distance). Let
the k nearest neighbors of ˜Xa (t ) be ˜Xa (t ′
α(t )), α = 1,..., k. (Note that the k
values t ′
α(t ) depend on t .) Then we can deﬁne the predicted value Y(t ) simply
as an average of the corresponding realized values Y (t ′
α(t )):
Y(t ) = 1
k
k∑
α=1
Y (t ′
α(t )) ( 3.74)
Alternatively, we can, e.g., consider a linear model
Y(t ) =
k∑
α=1
Y (t ′
α(t )) wα + v( 3.75)
and ﬁx the coefﬁcients wα and v by running a regression 25 of the realized
values Y (t ) over Y (t ′
α(t )) for some number—call it M—of values of t . I.e., we
pull Y (t ) for these values of t into an M-vector and regress it over the M × k
matrix of the corresponding values Y (t ′
α(t )). The coefﬁcients of this regression
are wα and v.
The advantage of using Eq. ( 3.74) is simplicity—there are no parameters
to train in this case. We still have to backtest the strategy (see below) out-
of-sample. The disadvantage is that equally weighting contributions of all k
nearest neighbors could be suboptimal. In this regard, there are various (e.g.,
distance-based) weighting schemes one may consider. Nontrivial weighting is
precisely what Eq. ( 3.75) intends to capture. However, this requires training
and cross-validation (using metrics such as root mean square error), and the
ﬁtted parameters w
α and v can be (and often are) out-of-sample unstable.
25We can run this regression without the intercept, in which case we only have the coefﬁcients wα,o r
with the intercept, in which case we also have the coefﬁcient v.

3 Stocks 59
The data can be split, e.g., 60% for training and 40% for cross-validation.
Ultimately, the strategy must backtest well out-of-sample.
The signal at t = 0 can be deﬁned using the predicted value Y = Y(0),
which is the expected return for the next T days. For single-stock trading 26
one can simply deﬁne thresholds for establishing long and short trades, and
liquidating existing positions, e.g., as follows 27:
Signal =
⎧
⎪⎪
⎪
⎨
⎪⎪
⎪
⎩
Establish long position if
Y > z1
Liquidate long position if Y ≤ z2
Establish short position if Y < −z1
Liquidate short position if Y ≥− z2
(3.76)
Here, z1 and z2 are trader-deﬁned thresholds. This signal must be backtested
out-of-sample. The number k of nearest neighbors can be optimized using a
backtest (by trying a set of values of k). Alternatively, one can use a common
heuristic, e.g., k = ﬂoor(√T∗) or k = ceiling(√T∗). Also see, e.g., (Hall et al.
2008).
3.18 Strategy: Statistical Arbitrage—Optimization
Let Cij be the sample or model covariance matrix for the N stock returns
i nap o r t f o l i o .28 Let Di be the dollar holdings in our portfolio. The expected
26Alternatively, one can use expected returns Yi computed for N stocks (where N ≫ 1) using a machine
learning algorithm as above and then use these expected returns in multi-stock cross-sectional strategies
such as mean-reversion/statistical arbitrage.
27For some literature on using machine learning for predicting stock returns, see, e.g., Adam and Lin
(2001), Ang and Quek ( 2006), Chen ( 2014), Chen et al. ( 2003), Creamer and Freund ( 2007, 2010),
Gestel et al. ( 2001), Grudnitski and Osborn ( 1993), Huang et al. ( 2005), Huang and Tsai ( 2009),
Huerta et al. ( 2013), Kablan ( 2009), Kakushadze and Yu ( 2016b, 2017c, 2018), Kara et al. ( 2011), Kim
(2003, 2006), Kim and Han ( 2000), Kordos and Cwiok ( 2011), Kryzanowski et al. ( 1993), Kumar and
Thenmozhi ( 2001), Liew and Mayster ( 2018), Lu et al. ( 2009), Milosevic ( 2016), Novak and Velušçek
(2016), Ou and Wang ( 2009), Refenes et al. ( 1994), Rodríguez-González et al. ( 2011), Saad et al. ( 1998),
Schumaker and Chen ( 2010), Subha and Nambi ( 2012), T ay and Cao ( 2001), T eixeira and de Oliveira
(2010), Tsai and Hsiao ( 2010), Vanstone and Finnie ( 2009), Yao and T an ( 2000), Yao et al. ( 1999), Yu
et al. ( 2005).
28The sample covariance matrix based on a time series of historical returns is singular if T ≤ N + 1,
where T is the number of observations in the time series. Even if it is nonsingular, unless T ≫ N ,w h i c h
is rarely (if ever) the case, the off-diagonal elements of the sample covariance matrix typically are unstable
out-of-sample. Therefore, in practice, typically a model covariance matrix (which is positive-deﬁnite and
should be sufﬁciently stable out-of-sample) is used (see below).

60 Z. Kakushadze and J. A. Serur
portfolio P&L P, volatility V and Sharpe ratio S are given by
P =
N∑
i =1
Ei Di (3.77)
V 2 =
N∑
i,j =1
Cij Di D j (3.78)
S = P/V (3.79)
Here Ei are the expected stock returns. Instead of the dollar holdings Di ,i t
is more convenient to work with dimensionless holding weights (which are
positive/negative for long/short positions)
wi = Di /I (3.80)
where I is the total investment level. The holding weights satisfy the condition
N∑
i =1
|wi | = 1 (3.81)
We have P = I × ˜P, V = I × ˜V and S = ˜P/˜V ,w h e r e
˜P =
N∑
i =1
Ei wi (3.82)
˜V 2 =
N∑
i,j =1
Cij wi wj (3.83)
T o determine the portfolio weights wi , often one requires that the Sharpe ratio
(Sharpe 1966, 1994) be maximized:
S → max (3.84)
Assuming no additional conditions on wi (e.g., upper or lower bounds), the
solution to Eq. ( 3.84) in the absence of trading costs is given by
wi = γ
N∑
j =1
C−1
ij E j (3.85)

3 Stocks 61
where C−1 is the inverse of C, and the normalization coefﬁcient γ is deter-
m i n e df r o mE q .( 3.81)( a n d γ> 0 so ˜P > 0). The weights given by
Eq. (3.85) generically do not correspond to a dollar-neutral portfolio. T o have
a dollar-neutral portfolio, we need to maximize the Sharpe ratio subject to the
dollar-neutrality constraint.
Dollar-Neutrality
We can achieve dollar-neutrality as follows. In the absence of bounds, trading
costs, etc., the Sharpe ratio is invariant under simultaneous rescalings of all
holding weights wi → ζw i ,w h e r eζ> 0. Due to this scale invariance, the
Sharpe ratio maximization problem can be recast in terms of minimizing a
quadratic objective function:
g(w, λ)= λ
2
N∑
i,j =1
Cij wi wj −
N∑
i =1
Ei wi (3.86)
g(w, λ)→ min (3.87)
where λ> 0 is a parameter, and minimization is w.r.t. wi . The solution is
given by
wi = 1
λ
N∑
j =1
C−1
ij E j (3.88)
and λ is ﬁxed via Eq. ( 3.81). The objective function approach—which is
the mean-variance optimization (Markowitz 1952)—is convenient if we wish
to impose linear homogeneous constraints (which do not spoil the aforesaid
scale invariance) on wi , e.g., the dollar-neutrality constraint. We introduce a
Lagrange multiplier μ29:
g(w, μ, λ)= λ
2
N∑
i,j =1
Cij wi wj −
N∑
i =1
Ei wi − μ
N∑
i =1
wi (3.89)
g(w, μ, λ)→ min (3.90)
29By introducing multiple Lagrange multipliers, we can have multiple linear homogeneous constraints
(see, e.g., Kakushadze 2015).

62 Z. Kakushadze and J. A. Serur
Minimization w.r.t. wi and μ now gives the following equations:
λ
N∑
j =1
Cij wj = Ei + μ( 3.91)
N∑
i =1
wi = 0 (3.92)
So we have dollar-neutrality. The solution to Eqs. ( 3.91)a n d( 3.92) is given by:
wi = 1
λ
⎡
⎣
N∑
j =1
C−1
ij E j −
N∑
j =1
C−1
ij
∑ N
k,l=1 C−1
kl El
∑ N
k,l=1 C−1
kl
⎤
⎦ (3.93)
By construction, wi satisfy the dollar-neutrality constraint ( 3.92), and λ is
ﬁxed via Eq. ( 3.81). The expected returns Ei can be based on mean-reversion,
momentum, machine learning or other signals. Equation ( 3.93)c o n s t r u c t sa
dollar-neutral portfolio with “risk management” built in. E.g., the weights wi
(roughly) are suppressed by stock volatilities σi (where σ2
i = Cii ) assuming
that on average |Ei | are of order σi .30
The above implementation of the dollar-neutrality constraint via minimiz-
ing the quadratic objective function ( 3.89) is equivalent to imposing this
constraint in Sharpe ratio maximization as no trading costs, position/trading
bounds, nonlinear/inhomogeneous constraints, etc., are present. More gener-
ally Sharpe ratio maximization is not equivalent to minimizing a quadratic
objective function (see, e.g., Kakushadze 2015), albeit in practice usually the
latter approach is used.
30T ypically,Cij is a multifactor risk model covariance matrix. For a general discussion, see, e.g., Grinold
and Kahn ( 2000). For explicit implementations (including source code), see, e.g., Kakushadze ( 2015d),
Kakushadze and Yu ( 2016a, 2017a). For multifactor models, the weights are approximately neutral w.r.t.
the columns of the factor loadings matrix. The exact neutrality is attained in the zero speciﬁc risk limit,
where optimization reduces to a weighted regression (see, e.g., Kakushadze 2015).

3 Stocks 63
3.19 Strategy: Market-Making
Over-simplistically, this strategy amounts to capturing the bid-ask spread for
a given stock and can be (again, over-simplistically) summarized as follows:
Rule =
{
Buy at the bid
Sell at the ask (3.94)
In a market where most order ﬂow is “dumb” (or uninformed), this strategy on
average would work very well. However, in a market where most order ﬂow is
“smart” (or informed, i.e., “toxic”), this strategy, as stated, would lose money.
This is because of adverse selection , where, precisely because most order ﬂow is
smart, most ﬁlls at the bid (ask) would be when the market is trading through
it downward (upward), so these trades would lose money. Furthermore, most
limit orders to buy (sell) at the bid (ask) would never be ﬁlled as the price
would run away from them, i.e., increase (decrease). So, ideally, this strategy
should be structured such that it captures dumb order ﬂow and avoids smart
order ﬂow, which is not that simple.
One approach is, at any given time, within a short time horizon, to stay on
the “right” side of the market, i.e., to have a short-horizon signal indicating
the direction of the market and place limit orders accordingly (to buy at the
bid if the signal indicates a price increase, and to sell at the ask if the signal
indicates a price decrease). If the signal were (magically) 100% correct, this
would capture the dumb order ﬂow assuming that the orders get ﬁlled. This
is a big assumption as for this to be guaranteed, the trader would have to be
#1 in the queue among many other market participants placing limit orders
at the same price point. This is where high frequency trading comes in—it is
essentially all about speed with which orders are placed, canceled, and cancel-
replaced. Infrastructure and technology are key in this.
Another possibility is to modulate the short-horizon signal with a longer-
horizon signal (which can still be an intraday signal). The longer-horizon signal
typically will have a higher cents-per-share
31 than the shorter-horizon signal.
Now certain trades can be proﬁtable even with adverse selection, because they
are established based on the longer-horizon signal. I.e., they “lose money”
in the short term due to adverse selection (as the market trades through the
corresponding limit orders), but they make money in a longer term. The
market-making aspect of this is valuable as placing a passive limit order as
31“Cents-per-share” is deﬁned as the realized P&L in cents (as opposed to dollars) divided by the total
shares traded (which includes both establishing and liquidating trades). Note that the longer-horizon
signal generally has a lower Sharpe ratio than the shorter-horizon signal.

64 Z. Kakushadze and J. A. Serur
opposed to an aggressive market or limit order saves money. On the other
hand, in some cases, if the longer-horizon signal is strong enough and the
shorter-horizon signal is in the same direction, a passive limit order would
likely not get ﬁlled and it may make more sense to place an aggressive order.
Such aggressive order ﬂow is not dumb but smart, as it is based on nontrivial
short- and long-horizon signals with a positive expected return.
32 And speed
still matters.
3.20 Strategy: Alpha Combos
With technological advances—hardware becoming cheaper and more
powerful—it is now possible to data mine hundreds of thousands and even
millions of alphas using machine learning methods. Here the term “alpha”—
following common trader lingo—generally means any reasonable “expected
return” that one may wish to trade on and is not necessarily the same as the
“academic” alpha.33 In practice, often the detailed information about how
alphas are constructed may not even be available, e.g., the only data available
could be the position data, so “alpha” then is a set of instructions to achieve cer-
tain stock (or some other instrument) holdings by some times t1,t2,... Also,
“machine learning” here refers to sophisticated methods that go beyond single-
stock methods such as those discussed in Sect. 3.17 and involve cross-sectional
analyses based on price-volume as well as other types of data (e.g., market cap,
some other fundamental data such as earnings, industry classiﬁcation data,
sentiment data, etc.) for a large number of stocks (typically, a few thousand
and up). 101 explicit examples of such quantitative trading alphas are given
32Dumb order ﬂow can come from, e.g., uninformed retail traders. It can also come from ultra-
long-horizon institutional traders (mutual funds, pension funds, etc.), whose outlook can be months
or years and who are not concerned about a few pennies’ worth of difference in the execution price on
short horizons (i.e., this is only “short-term dumb” order ﬂow). For a more detailed discussion, see, e.g.,
Kakushadze ( 2015c), Lo ( 2008). For some literature on high frequency trading and market-making, see,
e.g., Aldridge ( 2013), Anand and Venkataraman ( 2016), Avellaneda and Stoikov ( 2008), Baron et al.
(2014) ,B e n o se ta l .( 2017), Benos and Sagade ( 2016), Biais and Foucault ( 2014), Biais et al. ( 2014),
Bowen et al. ( 2010), Bozdog et al. ( 2011), Brogaard et al. ( 2014, 2015), Brogaard and Garriott ( 2018),
Budish et al. ( 2015), Carrion ( 2013), Carrion and Kolay ( 2017), Easley et al. ( 2011, 2012), Egginton
et al. ( 2016), Hagströmer and Nordén ( 2013), Hagströmer et al. ( 2014), Harris and Namvar ( 2016),
Hasbrouck and Saar ( 2013), Hendershott et al. ( 2011, 2013), Hendershott and Riordan ( 2013), Hirschey
(2018), Holden and Jacobsen ( 2014), Jarrow and Protter ( 2012), Khandani and Lo ( 2011), Kirilenko et al.
(2017), Korajczyk and Murphy ( 2017), Kozhan and Tham ( 2012), Li et al. ( 2014), Madhavan ( 2012),
Menkveld (2013), Menkveld ( 2016), Muthuswamy et al. ( 2011), O’Hara (2015), Pagnotta and Philippon
(2012), Riordan and Storkenmaier ( 2012), Van Kervel and Menkveld ( 2017
).
33By “academic” alpha we mean Jensen’s alpha (Jensen 1968) or a similar performance index.

3 Stocks 65
in Kakushadze ( 2016).34 The ﬂipside is that these ubiquitous alphas are faint,
ephemeral and cannot be traded on their own as any proﬁt on paper would be
eaten away by trading costs. T o mitigate this, one combines a large number of
such alphas and trades the so-combined “mega-alpha”. Hence “alpha combo”
strategies.
This is not critical, but for deﬁniteness let us assume that all alphas trade
the same underlying instruments, even more concretely, the same universe of
(say, 2500) most liquid U.S. stocks. Each alpha produces desired holdings
for this trading universe. What we need is the weights with which to com-
bine individual alphas, whose number N can be large (in hundreds of thou-
sands or even millions). 35 Here is a procedure for ﬁxing the alpha weights wi ,
i = 1,..., N (Kakushadze and Yu 2017b) (also see Kakushadze and Yu 2018):
• (1) Start with a time series of realized alpha returns36 Ris , i = 1,..., N ,
s = 1,..., M + 1.
• (2) Calculate the serially demeaned returns Xis = Ris − 1
M+1
∑ M+1
s=1 Ris .
• (3) Calculate sample variances of alpha returns 37 σ2
i = 1
M
∑ M+1
s=1 X 2
is .
• (4) Calculate the normalized demeaned returns Yis = Xis /σi .
• (5) Keep only the ﬁrst M columns in Yis : s = 1,..., M.
• (6) Cross-sectionally demean Yis : /Lambda1is = Yis − 1
N
∑ N
j =1 Y js .
• (7) Keep only the ﬁrst M − 1 columns in /Lambda1is : s = 1,..., M − 1.
• (8) T ake the expected alpha returns Ei and normalize them: ˜Ei = Ei /σi .
One (but by far not the only) way of computing expected alpha returns is
via d-day moving averages (note that d need not be the same as T ):
Ei = 1
d
d∑
s=1
Ris (3.95)
• (9) Calculate the residuals ˜εi of the regression (without the intercept and
with unit weights) of ˜Ei over /Lambda1is .
• (10) Set the alpha portfolio weights to wi = η˜εi /σi .
• (11) Set the normalization coefﬁcient η such that ∑ N
i =1 |wi | = 1.
34This is a secretive ﬁeld, so literature on this subject is very scarce. Also see, e.g., Kakushadze and
T ulchinsky (2016), T ulchinsky et al. ( 2015).
35Note that N here refers to the number of alphas, not the number of underlying stocks.
36Here s = 1,..., T = M + 1 labels the times ts , where, as before, t1 corresponds to the most recent
time (albeit the time direction is not crucial below), and the alpha returns are Ris = Ri (ts ). T ypically, the
alpha returns are computed daily, from close to close.
37Their normalization is immaterial in what follows.

66 Z. Kakushadze and J. A. Serur
3.21 A Few Comments
We end this section with a few comments on some of the stock trading strategies
discussed above. First, single-stock technical analysis strategies (i.e., those based
solely on single-stock as opposed to cross-sectional data) such as those based
on moving averages, support and resistance, channel and even single-stock
KNN, are deemed “unscientiﬁc” by many professionals and academics. On
the face of it, “fundamentally” speaking (not to be confused with fundamental
analysis), there is no reason why, say, a short moving average crossing a long
moving average should have any forecasting power.
38 This is not to say that
moving averages are “unscientiﬁc” or that they should not be used. After all,
e.g., trend following/momentum strategies are based on moving averages, i.e.,
the expected returns are computed via moving averages. However, looking
at a large cross-section of stocks brings in a statistical element into the game.
Mean-reversion is expected to work because stocks are expected to be correlated
if they belong to the same industries, etc. This relates back to fundamental
analysis and—even more importantly—to the investors’ perception of how
stock prices/returns “should” behave based on the companies’ fundamentals.
However, here too it is important to keep in mind that the stock market—an
imperfect man-made construct—is not governed by laws of nature the same
way as, say, the motion of planets in the solar system is governed by fundamental
laws of gravity (see, e.g., Kakushadze 2015c). The markets behave the way they
do because their participants behave in certain ways, which are sometimes
irrational and certainly not always efﬁcient. In this regard, the key difference
between technical analysis strategies and statistical arbitrage strategies is that
the latter are based on certain perceptions trickled down from longer holding
horizons (fundamental analysis based strategies) to shorter horizons (statistical
arbitrage) further enhanced by statistics, i.e., the fact that these strategies are
based on a large number of stocks whose properties are further “stratiﬁed”
according to some statistical and other features.
This brings us to the second point relating to precisely these “stratiﬁcations”
in the context of statistical arbitrage. Thus, in Sect. 3.10 we can use a binary
industry classiﬁcation matrix as the loadings matrix /Omega1
iA . Such industry classiﬁ-
cations are based on pertinent fundamental/economic data, such as companies’
products and services, revenue sources, suppliers, competitors, partners, etc.
They are essentially independent of the pricing data and, if well-built, tend
to be rather stable out-of-sample as companies seldom jump industries. How-
ever, binary classiﬁcations can also be built based purely on pricing data, via
38Arguendo, the momentum effect may appear to provide a basis for such forecasting power in some cases.
However, then one could argue, e.g., that these are momentum strategies in disguise.

3 Stocks 67
clustering algorithms (see, e.g., Kakushadze and Yu 2016b). Alternatively, the
matrix /Omega1iA can be non-binary and built using, say, principal components (see,
e.g., Kakushadze and Yu 2017a). Some of the columns of /Omega1iA can be based
on longer-horizon style risk factors such as value, growth, size, momentum,
liquidity and volatility (see, e.g., Ang et al. 2006;A n s o n2013; Asness 1995;
Asness et al. 2000, 2001;B a n z1981;B a s u1977; Fama and French 1992, 1993;
Haugen 1995; Jegadeesh and Titman 1993; Lakonishok et al. 1994;L i e wa n d
Vassalou 2000; Pástor and Stambaugh 2003; Scholes and Williams 1977),39
or shorter-horizon style factors (Kakushadze 2015b).
References
Adam, F ., & Lin, L. H. (2001). An Analysis of the Applications of Neural Networks
in Finance. Interfaces, 31(4), 112–122.
Aldridge, I. (2013). High-Frequency T rading: A Practical Guide to Algorithmic Strategies
and T rading Systems (2nd ed.). Hoboken, NJ: Wiley.
Altman, N. S. (1992). An Introduction to Kernel and Nearest-Neighbor Nonpara-
metric Regression. American Statistician, 46 (3), 175–185.
Amenc, N., Ducoulombier, F ., Goltz, F ., & Ulahel, J. (2016). T en Misconceptions
about Smart Beta (Working Paper). Available online: https://www.edhec.edu/sites/
www.edhec-portail.pprod.net/ﬁles/publications/pdf/edhec-position-paper-ten-
misconceptions-about-smart-beta%5F1468395239135-pdfjpg .
Amenc, N., Goltz, F ., Sivasubramanian, S., & Lodh, A. (2015). Robustness of Smart
Beta Strategies. Journal of Index Investing , 6 (1), 17–38.
Amihud, Y. (2002). Illiquidity and Stock Returns: Cross-Section and Time-Series
Effects. Journal of Financial Markets , 5 (1), 31–56.
Amiri, M., Zandieh, M., Vahdani, B., Soltani, R., & Roshanaei, V . (2010). An Inte-
grated Eigenvector-DEA-TOPSIS Methodology for Portfolio Risk Evaluation in
the FOREX Spot Market. Expert Systems with Applications , 37 (1), 509–516.
Anand, A., & Venkataraman, K. (2016). Market Conditions, Fragility, and the Eco-
nomics of Market Making. Journal of Financial Economics , 121(2), 327–349.
An, B.-J., Ang, A., Bali, T . G., & Cakici, N. (2014). The Joint Cross Section of Stocks
and Options. Journal of Finance , 69 (5), 2279–2337.
Andrade, G., Mitchell, M., & Stafford, E. (2001). New Evidence and Perspectives on
Mergers. Journal of Economic Perspectives , 15 (2), 103–120.
Andrie¸s, A. M., & Vîrlan, C. A. (2017). Risk Arbitrage in Emerging Europe: Are
Cross-Border Mergers and Acquisition Deals More Risky? Economic Research—
Ekonomska Istraživanja, 30(1), 1367–1389.
Ang, A., Hodrick, R., Xing, Y., & Zhang, X. (2006). The Cross-Section of Volatility
and Expected Returns. Journal of Finance , 61(1), 259–299.
39For (il)liquidity related considerations, also see, e.g., Amihud ( 2002).

68 Z. Kakushadze and J. A. Serur
Ang, A., Hodrick, R., Xing, Y., & Zhang, X. (2009). High Idiosyncratic Volatility
and Low Returns: International and Further U.S. Evidence. Journal of Financial
Economics, 91(1), 1–23.
Ang, K. K., & Quek, C. (2006). Stock T rading Using RSPOP: A Novel Rough Set-
Based Neuro-Fuzzy Approach. IEEE T ransactions on Neural Networks, 17 (5), 1301–
1315.
Anson, M. (2013). Performance Measurement in Private Equity: The Impact of FAS
157 on the Lagged Beta Effect. Journal of Private Equity , 17 (1), 29–44.
Antonacci, G. (2017). Risk Premia Harvesting Through Dual Momentum. Journal of
Management & Entrepreneurship , 11(1), 27–55.
Arnott, R. D., Hsu, J., Kalesnik, V ., & Tindall, P . (2013). The Surprising Alpha from
Malkiel’s Monkey and Upside-Down Strategies. Journal of Portfolio Management ,
39 (4), 91–105.
Asem, E., & Tian, G. (2010). Market Dynamics and Momentum Proﬁts. Journal of
Financial and Quantitative Analysis , 45 (6), 1549–1562.
Asness, C. S. (1994). Variables that Explain Stock Returns. Ph.D. thesis, University of
Chicago, Chicago, IL.
Asness, C. S. (1995). The Power of Past Stock Returns to Explain Future Stock Returns
(Working Paper, Unpublished). New York, NY: Goldman Sachs Asset Manage-
ment.
Asness, C. S., Porter, R. B., & Stevens, R. L. (2000). Predicting Stock Returns Using
Industry-Relative Firm Characteristics (Working Paper). Available online: https://
ssrn.com/abstract=213872.
Asness, C. S. (1997). The Interaction of Value and Momentum Strategies. Financial
Analysts Journal, 53(2), 29–36.
Asness, C. S., Frazzini, A., Israel, R., & Moskowitz, T . (2014). Fact, Fiction, and
Momentum Investing. Journal of Portfolio Management , 40(5), 75–92.
Asness, C. S., Krail, R. J., & Liew, J. M. (2001). Do Hedge Funds Hedge? Journal of
Portfolio Management, 28(1), 6–19.
Asness, C. S., Moskowitz, T ., & Pedersen, L. H. (2013). Value and Momentum
Everywhere. Journal of Finance , 68(3), 929–985.
Avellaneda, M., & Lee, J. H. (2010). Statistical Arbitrage in the U.S. Equity Market.
Quantitative Finance , 10(7), 761–782.
Avellaneda, M., & Stoikov, S. (2008). High Frequency T rading in a Limit Order Book.
Quantitative Finance , 8(3), 217–224.
Baker, M., Bradley, B., & Wurgler, J. (2011). Benchmarks as Limits to Arbitrage:
Understanding the Low-Volatility Anomaly. Financial Analysts Journal , 67 (1), 40–
54.
Baker, M., Pan, A., & Wurgler, J. (2012). The Effect of Reference Point Prices on
Mergers and Acquisitions. Journal of Financial Economics , 106 (1), 49–71.
Baker, M., & Sava¸soglu, S. (2002). Limited Arbitrage in Mergers and Acquisitions.
Journal of Financial Economics , 64 (1), 91–115.
Bali, T . G., & Hovakimian, A. (2009). Volatility Spreads and Expected Stock Returns.
Management Science , 55 (11), 1797–1812.

3 Stocks 69
Banz, R. (1981). The Relationship Between Return and Market Value of Common
Stocks. Journal of Financial Economics , 9 (1), 3–18.
Barber, J., Bennett, S., & Gvozdeva, E. (2015). How to Choose a Strategic Multifactor
Equity Portfolio? Journal of Index Investing , 6 (2), 34–45.
Baron, M., Brogaard, J., Hagströmer, B., & Kirilenko, A. (2014). Risk and Return
in High-Frequency T rading. Journal of Financial and Quantitative Analysis (forth-
coming). Available online: https://ssrn.com/abstract=2433118.
Barroso, P ., & Santa-Clara, P . (2014). Momentum Has Its Moments. Journal of Finan-
cial Economics , 116 (1), 111–120.
Bartov, E., Radhakrishnan, S., & Krinsky, I. (2005). Investor Sophistication and Pat-
terns in Stock Returns after Earnings Announcements. Accounting Review , 75 (1),
289–319.
Basu, S. (1977). The Investment Performance of Common Stocks in Relation to
Their Price to Earnings Ratios: A T est of the Efﬁcient Market Hypothesis. Journal
of Finance , 32(3), 663–682.
Battalio, R., & Mendenhall, R. (2007). Post-Earnings Announcement Drift: Intra-Day
Timing and Liquidity Costs (Working Paper). Available online: https://ssrn.com/
abstract=937257.
Batten, J., & Ellis, C. (1996). T echnical T rading System Performance in the Australian
Share Market: Some Empirical Evidence. Asia Paciﬁc Journal of Management , 13(1),
87–99.
Benos, E., Brugler, J., Hjalmarsson, E., & Zikes, F . (2017). Interactions Among High-
Frequency T raders. Journal of Financial and Quantitative Analysis , 52(4), 1375–
1402.
Benos, E., & Sagade, S. (2016). Price Discovery and the Cross-Section of High-
Frequency T rading.Journal of Financial Markets , 30, 54–77.
BenZion, U., Klein, P ., Shachmurove, Y., & Yagil, J. (2003). Efﬁciency Differences
Between the S&P 500 and the T el-Aviv 25 Indices: A Moving Average Comparison.
International Journal of Business , 8(3), 267–284.
Bernard, V . L., & Thomas, J. K. (1989). Post-Earnings-Announcement Drift: Delayed
Price Response or Risk Premium? Journal of Accounting Research , 27, 1–36.
Bernard, V . L., & Thomas, J. K. (1990). Evidence That Stock Prices Do Not Fully
Reﬂect the Implications of Current Earnings for Future Earnings. Journal of
Accounting and Economics , 13(4), 305–340.
Bester, A., Martinez, V . H., & Rosu, I. (2017). Cash Mergers and the Volatility Smile
(Working Paper). Available online: https://ssrn.com/abstract=1364491.
Bhojraj, S., & Swaminathan, B. (2006). Macromomentum: Returns Predictability in
International Equity Indices. Journal of Business , 79 (1), 429–451.
Bhushan, R. (1994). An Informational Efﬁciency Perspective on the Post-Earnings
Announcement Drift. Journal of Accounting and Economics , 18(1), 45–65.
Biais, B., & Foucault, T . (2014). HFT and Market Quality. Bankers, Markets &
Investors, 128, 5–19.
Biais, B., Foucault, T ., & Moinas, S. (2014). Equilibrium Fast T rading (Working
Paper). Available online: https://ssrn.com/abstract=2024360.

70 Z. Kakushadze and J. A. Serur
Birari, A., & Rode, M. (2014). Edge Ratio of Nifty for Last 15 Years on Donchian
Channel. SIJ T ransactions on Industrial, Financial & Business Management (IFBM) ,
2(5), 247–254.
Black, F . (1972). Capital Market Equilibrium with Restricted Borrowing. Journal of
Business, 45 (3), 444–455.
Black, F ., & Litterman, R. (1991). Asset Allocation: Combining Investors’ Views with
Market Equilibrium. Journal of Fixed Income , 1(2), 7–18.
Black, F ., & Litterman, R. (1992). Global Portfolio Optimization. Financial Analysts
Journal, 48(5), 28–43.
Blitz, D. C., Huij, J., Lansdorp, S., & Verbeek, M. (2013). Short-T erm Residual
Reversal. Journal of Financial Markets , 16 (3), 477–504.
Blitz, D. C., Huij, J., & Martens, M. (2011). Residual Momentum. Journal of Empir-
ical Finance , 18(3), 506–521.
Blitz, D. C., & van Vliet, P . (2007). The Volatility Effect: Lower Risk without Lower
Return. Journal of Portfolio Management , 34 (1), 102–113.
Bogomolov, T . (2013). Pairs T rading Based on Statistical Variability of the Spread
Process. Quantitative Finance , 13(9), 1411–1430.
Bollen, N. P . B., & Whaley, R. (2004). Does Net Buying Pressure Affect the Shape of
Implied Volatility Functions? Journal of Finance , 59 (2), 711–754.
Boudoukh, J., Richardson, M., & Whitelaw, R. F . (1994). Industry Returns and the
Fisher Effect. Journal of Finance , 49 (5), 1595–1615.
Bowen, D. A., & Hutchinson, M. C. (2016). Pairs T rading in the UK Equity Market:
Risk and Return. European Journal of Finance , 22(14), 1363–1387.
Bowen, D. A., Hutchinson, M. C., & O’Sullivan, N. (2010). High Frequency Equity
Pairs T rading: T ransaction Costs, Speed of Execution and Patterns in Returns.
Journal of T rading, 5 (3), 31–38.
Bozdog, D., Florescu, I., Khashanah, K., & Wang, J. (2011). Rare Events Analysis of
High-Frequency Equity Data. Wilmott Magazine , 54, 74–81.
Brock, W ., Lakonishock, J., & LeBaron, B. (1992). Simple T echnical T rading Rules
and the Stochastic Properties of Stock Returns. Journal of Finance , 47 (5), 1731–
1764.
Brogaard, J., & Garriott, C. (2018). High-Frequency T rading Competition (Working
Paper). Available online: https://ssrn.com/abstract=2435999.
Brogaard, J., Hagströmer, B., Nordén, L., & Riordan, R. (2015). T rading Fast and
Slow: Colocation and Liquidity. Review of Financial Studies , 28(12), 3407–3443.
Brogaard, J., Hendershott, T ., & Riordan, R. (2014). High-Frequency T rading and
Price Discovery. Review of Financial Studies , 27
(8), 2267–2306.
Brown, K. C., & Raymond, M. V . (1986). Risk Arbitrage and the Prediction of
Successful Corporate T akeovers. Financial Management , 15 (3), 54–63.
Budish, E., Cramton, P ., & Shim, J. (2015). The High-Frequency T rading Arms
Race: Frequent Batch Auctions as a Market Design Response. Quarterly Journal of
Economics, 130(4), 1547–1621.

3 Stocks 71
Busch, T ., Christensen, B. J., & Nielsen, M. Ø. (2011). The Role of Implied Volatility
in Forecasting Future Realized Volatility and Jumps in Foreign Exchange, Stock,
and Bond Markets. Journal of Econometrics , 160(1), 48–57.
Caldeira, J., & Moura, G. V . (2013). Selection of a Portfolio of Pairs Based on Cointe-
gration: A Statistical Arbitrage Strategy (Working Paper). Available online: https://
ssrn.com/abstract=2196391.
Cao, C., Goldie, B., Liang, B., & Petrasek, L. (2016). What Is the Nature of Hedge
Fund Manager Skills? Evidence from the Risk-Arbitrage Strategy. Journal of Finan-
cial and Quantitative Analysis , 51(3), 929–957.
Carhart, M. M. (1997). Persistence in Mutual Fund Performance. Journal of Finance ,
52(1), 57–82.
Carrion, A. (2013). Very Fast Money: High-Frequency T rading on the NASDAQ.
Journal of Financial Markets , 16 (4), 680–711.
Carrion, A., & Kolay, M. (2017). T rade Signing in Fast Markets (Working Paper).
Available online: https://ssrn.com/abstract=2489868.
Chakravarty, S., Gulen, H., & Mayhew, S. (2004). Informed T rading in Stock and
Option Markets. Journal of Finance , 59 (3), 1235–1257.
Chang, R. P ., Ko, K.-C., Nakano, S., & Rhee, S. G. (2016). Residual Momentum
and Investor Underreaction in Japan (Working Paper). Available online: http://sfm.
ﬁnance.nsysu.edu.tw/php/Papers/CompletePaper/134-1136665035.pdf .
Chan, K. C., Jegadeesh, N., & Lakonishok, J. (1996). Momentum Strategies. Journal
of Finance , 51(5), 1681–1713.
Chaves, D. B. (2012). Eureka! A Momentum Strategy That alsoWorks in Japan (Working
Paper). Available online: https://ssrn.com/abstract=1982100.
Chen, H. J., Chen, S. J., Chen, Z., & Li, F . (2017). Empirical Investigation of an
Equity Pairs T rading Strategy.Management Science (forthcoming). https://doi.org/
10.1287/mnsc.2017.2825.
Chen, M. Y. (2014). A High-Order Fuzzy Time Series Forecasting Model for Internet
Stock T rading.Future Generation Computer Systems , 37, 461–467.
Chen, T . F ., Chung, S. L., & Tsai, W . C. (2016). Option-Implied Equity Risk and
the Cross-Section of Stock Returns. Financial Analysts Journal , 72(6), 42–55.
Chen, A. S., Leung, M. T ., & Daouk, H. (2003). Application of Neural Networks to
an Emerging Financial Market: Forecasting and T rading the T aiwan Stock Index.
Computers & Operations Research , 30(6), 901–923.
Cheung, W . (2010). The Black-Litterman Model Explained. Journal of Asset Manage-
ment, 11(4), 229–243.
Chin, J. Y. F ., Prevost, A. K., & Gottesman, A. A. (2002). Contrarian Investing in
a Small Capitalization Market: Evidence from New Zealand. Financial Review ,
37 (3), 421–446.
Chordia, T ., Goyal, A., Sadka, G., Sadka, R., & Shivakumar, L. (2009). Liquidity and
the Post-Earnings-Announcement Drift. Financial Analysts Journal
, 65 (4), 18–32.
Chordia,T ., & Shivakumar, L. (2002). Momentum, Business Cycle, andTime-Varying
Expected Returns. Journal of Finance , 57 (2), 985–1019.

72 Z. Kakushadze and J. A. Serur
Chordia, T ., & Shivakumar, L. (2006). Earnings and Price Momentum. Journal of
Financial Economics , 80(3), 627–656.
Chuang, H. (2015). Time Series Residual Momentum (Working Paper). Available
online: http://www.econ.tohoku.ac.jp/econ/datascience/DDSR-DP/no38.pdf .
Chuang, H., & Ho, H.-C. (2014). Implied Price Risk and Momentum Strategy.
Review of Finance , 18(2), 591–622.
Clarke, R. G., de Silva, H., & Thorley, S. (2006). Minimum-Variance Portfolios in
the U.S. Equity Market. Journal of Portfolio Management , 33(1), 10–24.
Clarke, R. G., de Silva, H., & Thorley, S. (2010). Know Your VMS Exposure. Journal
of Portfolio Management , 36 (2), 52–59.
Cochrane, J. H. (1999). Portfolio Advice for a Multifactor World. Federal Reserve Bank
of Chicago, Economic Perspectives , 23(3), 59–78.
Conrad, J., Dittmar, R. F ., & Ghysels, E. (2013). Ex Ante Skewness and Expected
Stock Returns. Journal of Finance , 68(1), 85–124.
Conrad, J., & Kaul, G. (1998). An Anatomy of T rading Strategies. Review of Financial
Studies, 11(3), 489–519.
Cooper, M. J., Gutierrez, R. C., Jr., & Hameed, A. (2004). Market States and Momen-
tum. Journal of Finance , 59 (3), 1345–1365.
Cornelli, F ., & Li, D. D. (2002). Risk Arbitrage in T akeovers. Review of Financial
Studies, 15 (3), 837–868.
Creamer, G. G., & Freund, Y. (2007). A Boosting Approach for Automated T rading.
Journal of T rading, 2(3), 84–96.
Creamer, G. G., & Freund, Y. (2010). Automated T rading with Boosting and Expert
Weighting. Quantitative Finance , 10(4), 401–420.
Cremers, M., & Weinbaum, D. (2010). Deviations from Put-Call Parity and Stock
Return Predictability. Journal of Financial and Quantitative Analysis , 45 (2), 335–
367.
Czaja, M.-G., Kaufmann, P ., & Scholz, H. (2013). Enhancing the Proﬁtability of
Earnings Momentum Strategies: The Role of Price Momentum, Information Dif-
fusion and Earnings Uncertainty. Journal of Investment Strategies , 2(4), 3–57.
Da Silva, A. S., Lee, W ., & Pornrojnangkool, B. (2009). The Black-Litterman Model
for Active Portfolio Management. Journal of Portfolio Management , 35 (2), 61–70.
Daniel, K. (2001). The Power and Size of Mean Reversion T ests. Journal of Empirical
Finance, 8(5), 493–535.
Daniel, K., & Moskowitz, T . J. (2016). Momentum Crashes. Journal of Financial
Economics, 122(2), 221–247.
De Zwart, G., Markwat, T ., Swinkels, L., & van Dijk, D. (2009). The Economic
Value of Fundamental and T echnical Information in Emerging Currency Markets.
Journal of International Money and Finance , 28(4), 581–604.
Dempster, M. A. H., & Jones, C. M. (2002). Can Channel Pattern T rading be Prof-
itably Automated? European Journal of Finance , 8(3), 275–301.
Doan, M. P ., Alexeev, V ., & Brooks, R. (2014). Concurrent Momentum and Contrar-
ian Strategies in the Australian Stock Market. Australian Journal of Management ,
41(1), 77–106.

3 Stocks 73
Do, B., & Faff, R. (2010). Does Simple Pairs T rading Still Work? Financial Analysts
Journal, 66 (4), 83–95.
Do, B., & Faff, R. (2012). Are Pairs T rading Proﬁts Robust to T rading Costs? Journal
of Financial Research , 35 (2), 261–287.
Donchian, R. D. (1960). High Finance in Copper. Financial Analysts Journal , 16 (6),
133–142.
Doyle, J. T ., Lundholm, R. J., & Soliman, M. T . (2006). The Extreme Future Stock
Returns Following I/B/E/S Earnings Surprises. Journal of Accounting Research ,
44 (5), 849–887.
Drobetz, W . (2001). How to Avoid the Pitfalls in Portfolio Optimization? Putting the
Black-Litterman Approach at Work. Financial Markets and Portfolio Management ,
15 (1), 59–75.
Dukes, W . P ., Frolich, C. J., & Ma, C. K. (1992). Risk Arbitrage in T ender Offers.
Journal of Portfolio Management , 18(4), 47–55.
Dzikeviˇcius, A., & Šanranda, S. (2010). EMA Versus SMA: Usage to Forecast Stock
Markets: The Case of S&P 500 and OMX Baltic Benchmark. Verslas: teorija ir
praktika—Business: Theory and Practice , 11(3), 248–255.
Easley, D., López de Prado, M. M., & O’Hara, M. (2011). The Microstructure of
the ‘Flash Crash’: Flow T oxicity, Liquidity Crashes and the Probability of Informed
Tr a d i n g .Journal of Portfolio Management , 37 (2), 118–128.
Easley, D., López de Prado, M. M., & O’Hara, M. (2012). The Volume Clock: Insights
into the High Frequency Paradigm. Journal of Portfolio Management , 39 (1), 19–29.
Edwards, R., & Magee, J. (1992). T echnical Analysis of Stock T rends .N e wY o r k ,N Y :
New York Institute of Finance.
Egginton, J. F ., Van Ness, B. F ., & Van Ness, R. A. (2016). Quote Stufﬁng. Financial
Management, 45 (3), 583–608.
Elder, A. (2014). The New T rading for a Living . Hoboken, NJ: Wiley.
Elliott, R. J., van der Hoek, J., & Malcolm, W . P . (2005). Pairs T rading. Quantitative
Finance, 5 (3), 271–276.
Engle, R. F ., & Granger, C. W . J. (1987). Co-integration and Error Correction:
Representation, Estimation and T esting. Econometrica, 55 (2), 251–276.
Erb, C., & Harvey, C. (2006). The Strategic and T actical Value of Commodity Futures.
Financial Analysts Journal , 62(2), 69–97.
Faber, M. (2007). A Quantitative Approach to T actical Asset Allocation. Journal of
Wealth Management, 9 (4), 69–79.
Fama, E. F . (1996). Multifactor Portfolio Efﬁciency and Multifactor Asset Pricing.
Journal of Financial and Quantitative Analysis , 31(4), 441–465.
Fama, E. F ., & French, K. R. (1992). The Cross-Section of Expected Stock Returns.
Journal of Finance , 47 (2), 427–465.
Fama, E. F ., & French, K. R. (1993). Common Risk Factors in the Returns on Stocks
and Bonds. Journal of Financial Economics , 33(1), 3–56.
Fama, E. F ., & French, K. R. (1996). Multifactor Explanations of Asset Pricing Anoma-
lies. Journal of Finance , 51(1), 55–84.

74 Z. Kakushadze and J. A. Serur
Fama, E. F ., & French, K. R. (1998). Value Versus Growth:The International Evidence.
Journal of Finance , 53(6), 1975–1999.
Fama, E. F ., & French, K. R. (2012). Size, Value and Momentum in International
Stock Returns. Journal of Financial Economics , 105 (3), 457–472.
Félix, J. A., & Rodríguez, F . F . (2008). Improving Moving Average T rading Rules with
Boosting and Statistical Learning Methods. Journal of Forecasting, 27 (5), 433–449.
Fiﬁeld, S. G. M., Power, D. M., & Knipe, D. G. S. (2008). The Performance of
Moving Average Rules in Emerging Stock Markets. Applied Financial Economics ,
18(19), 1515–1532.
Fisher, G., Shah, R., & Titman, S. (2016). Combining Value and Momentum. Journal
of Investment Management , 14 (2), 33–48.
Fong, W . M., & Yong, L. H. M. (2005). Chasing T rends: Recursive Moving Average
T rading Rules and Internet Stocks. Journal of Empirical Finance , 12(1), 43–76.
Foster, G., Olsen, C., & Shevlin, T . (1984). Earnings Releases, Anomalies, and the
Behavior of Security Returns. Accounting Review , 59 (4), 574–603.
Frazzini, A., & Pedersen, L. H. (2014). Betting Against Beta. Journal of Financial
Economics, 111(1), 1–25.
Fu, F . (2009). Idiosyncratic Risk and the Cross-Section of Expected Stock Returns.
Journal of Financial Economics , 91(1), 24–37.
Garcia-Feijóo, L., Kochard, L., Sullivan, R. N., & Wang, P . (2015). Low-Volatility
Cycles: The Inﬂuence of Valuation and Momentum on Low-Volatility Portfolios.
Financial Analysts Journal , 71(3), 47–60.
Garzarelli, F ., Cristelli, M., Pompa, G., Zaccaria, A., & Pietronero, L. (2014). Memory
Effects in Stock Price Dynamics: Evidences of T echnical T rading. Scientiﬁc Reports ,
4, 4487.
Gatev, E., Goetzmann, W . N., & Rouwenhorst, K. G. (2006). Pairs T rading: Per-
formance of a Relative-Value Arbitrage Rule. Review of Financial Studies , 19 (3),
797–827.
Géczy, C. C., & Samonov, M. (2016). T wo Centuries of Price-Return Momentum.
Financial Analysts Journal , 72(5), 32–56.
Gençay, R. (1996). Nonlinear Prediction of Security Returns with Moving Average
Rules. Journal of Forecasting , 15 (3), 165–174.
Gençay, R. (1998). The Predictability of Securities Returns with Simple T echnical
Rules. Journal of Empirical Finance , 5 (4), 347–359.
Gençay, R., & Stengos, T . (1998). Moving Average Rules, Volume and the Predictabil-
ity of Security Returns with Feedforward Networks. Journal of Forecasting, 17 (5–6),
401–414.
Gerakos, J., & Linnainmaa, J. (2012). Decomposing Value (Working Paper). Available
online: https://ssrn.com/abstract=2083166.
Gestel,T ., Suykens, J. A. K., Baestaend, D. E., Lambrechts, A., Lanckriet, G., Vandaele,
B., et al. (2001). Financial Time Series Prediction Using Least Squares Support
Vector Machines Within the Evidence Framework. IEEE T ransactions on Neural
Networks, 12(4), 809–821.

3 Stocks 75
Glabadanidis, P . (2015). Market Timing with Moving Averages. International Review
of Finance , 15 (3), 387–425.
Grifﬁn, J. M., Ji, X., & Martin, J. S. (2003). Momentum Investing and Business Cycle
Risks: Evidence from Pole to Pole. Journal of Finance , 58(6), 2515–2547.
Grinblatt, M., & Moskowitz, T . J. (2004). Predicting Stock Price Movements from
Past Returns: The Role of Consistency and T ax-Loss Selling. Journal of Financial
Economics, 71(3), 541–579.
Grinold, R. C., & Kahn, R. N. (2000). Active Portfolio Management .N e wY o r k ,N Y :
McGraw-Hill.
Grudnitski, G., & Osborn, L. (1993). Forecasting S&P and Gold Futures Prices: An
Application of Neural Networks. Journal of Futures Markets , 13(6), 631–643.
Grundy, B. D., & Martin, J. S. (2001). Understanding the Nature of the Risks and
the Source of the Rewards to Momentum Investing. Review of Financial Studies ,
14 (1), 29–78.
Gunasekarage, A., & Power, D. M. (2001). The Proﬁtability of Moving Average
T rading Rules in South Asian Stock Markets. Emerging Markets Review , 2(1), 17–
33.
Gutierrez, R. C., & Prinsky, C. A. (2007). Momentum, Reversal, and the T rading
Behaviors of Institutions. Journal of Financial Markets , 10(1), 48–75.
Hagströmer, B., & Nordén, L. (2013). The Diversity of High-Frequency T raders.
Journal of Financial Markets , 16 (4), 741–770.
Hagströmer, B., Nordén, L., & Zhang, D. (2014). The Aggressiveness of High-
Frequency T raders.Financial Review , 49 (2), 395–419.
Hall, P ., Park, B. U., & Samworth, R. J. (2008). Choice of Neighbor Order in Nearest-
Neighbor Classiﬁcation. Annals of Statistics , 36 (5), 2135–2152.
Hall, J., Pinnuck, M., & Thorne, M. (2013). Market Risk Exposure of Merger Arbi-
trage in Australia. Accounting & Finance , 53(1), 185–215.
Hardy, C. C. (1978). The Investor’s Guide to T echnical Analysis .N e wY o r k ,N Y :
McGraw-Hill.
Harford, J. (2005). What Drives Merger Waves? Journal of Financial Economics , 77 (3),
529–560.
Harris, L. E., & Namvar, E. (2016). The Economics of Flash Orders and T rading.
Journal of Investment Management , 14 (4), 74–86.
Hasbrouck, J., & Saar, G. (2013). Low-Latency T rading. Journal of Financial Markets ,
16 (4), 646–679.
Haugen, R. A. (1995). The New Finance: The Case Against Efﬁcient Markets . Upper
Saddle River, NJ: Prentice Hall.
Hendershott, T ., Jones, C., & Menkveld, A. (2011). Does Algorithmic T rading
Improve Liquidity? Journal of Finance , 66 (1), 1–33.
Hendershott, T ., Jones, C., & Menkveld, A. (2013). Implementation Shortfall with
T ransitory Price Effects. In D. Easley, M. López de Prado, & M. O’Hara (Eds.),
High Frequency T rading: New Realities for T raders, Markets and Regulators (Chapter
9). London, UK: Risk Books.

76 Z. Kakushadze and J. A. Serur
Hendershott, T ., & Riordan, R. (2013). Algorithmic T rading and the Market for
Liquidity. Journal of Financial and Quantitative Analysis , 48(4), 1001–1024.
Hew, D., Skerratt, L., Strong, N., & Walker, M. (1996). Post-Earnings-
Announcement Drift: Some Preliminary Evidence for the UK. Accounting & Busi-
ness Research , 26 (4), 283–293.
Hirschey, N. (2018). Do High-Frequency T raders Anticipate Buying and Selling Pressure?
(Working Paper). Available online: https://ssrn.com/abstract=2238516.
Hirshleifer, D., Lim, S. S., & T eoh, S. H. (2009). Driven to Distraction: Extraneous
Events and Underreaction to Earnings News. Journal of Finance , 64 (5), 2289–
2325.
Hodges, S., & Carverhill, A. (1993). Quasi Mean Reversion in an Efﬁcient Stock Mar-
ket: The Characterization of Economic Equilibria which Support Black-Scholes
Option Pricing. Economic Journal, 103(417), 395–405.
Holden, C. W ., & Jacobsen, S. (2014). Liquidity Measurement Problems in Fast
Competitive Markets: Expensive and Cheap Solutions. Journal of Finance , 69 (4),
1747–1885.
Hsieh, J., & Walkling, R. A. (2005). Determinants and Implications of Arbitrage
Holdings in Acquisitions. Journal of Financial Economics , 77 (3), 605–648.
Hsu, Y.-C., Lin, H.-W . and Vincent, K. (2018). Analyzing the Performance
of Multi-factor Investment Strategies Under Multiple T esting Framework
(Working Paper). Available online: http://www.econ.sinica.edu.tw/UpFiles/
2013092817175327692/Seminar_PDF2013093010102890633/17-A0001(all).
pdf .
Huang, W ., Nakamori, Y., & Wang, S.-Y. (2005). Forecasting Stock Market Movement
Direction with Support Vector Machine. Computers & Operation Research , 32(10),
2513–2522.
Huang, C. L., & Tsai, C. Y. (2009). A Hybrid SOFM-SVR with a Filter-Based Feature
Selection for Stock Market Forecasting. Expert Systems with Applications , 36 (2),
1529–1539.
Huck, N. (2009). Pairs Selection and Outranking: An Application to the S&P 100
Index. European Journal of Operational Research , 196 (2), 819–825.
Huck, N. (2015). Pairs T rading: Does Volatility Timing Matter? Applied Economics ,
47 (57), 6239–6256.
Huck, N., & Afawubo, K. (2014). Pairs T rading and Selection Methods: Is Cointe-
gration Superior? Applied Economics , 47 (6), 599–613.
Huerta, R., Elkan, C., & Corbacho, F . (2013). Nonlinear Support Vector Machines
Can Systematically Identify Stocks with High and Low Future Returns. Algorithmic
Finance, 2(1), 45–58.
Hühn, H., & Scholz, H. (2017). Alpha Momentum and Price Momentum (Working
Paper). Available online: https://ssrn.com/abstract=2287848.
Huij, J., & Lansdorp, S. (2017). Residual Momentum and Reversal Strategies Revisited
(Working Paper). Available online: https://ssrn.com/abstract=2929306.

3 Stocks 77
Hung, N. H. (2016). Various Moving Average Convergence Divergence T rading
Strategies: A Comparison. Investment Management and Financial Innovations ,
13(2), 363–369.
Hutson, E. (2000). T akeover T argets and the Probability of Bid Success: Evidence from
the Australian Market. International Review of Financial Analysis , 9 (1), 45–65.
Hwang, C.-Y., & George, T . J. (2004). The 52-Week High and Momentum Investing.
Journal of Finance , 59 (5), 2145–2176.
Idzorek, T . (2007). A Step-by-Step Guide to the Black-Litterman Model. In S. Satchell
(Ed.), Forecasting Expected Returns in the Financial Markets . Waltham, MA: Aca-
demic Press.
Jacobs, H., & Weber, M. (2015). On the Determinants of Pairs T rading Proﬁtability.
Journal of Financial Markets , 23, 75–97.
James, F . E., Jr. (1968). Monthly Moving Averages—An Effective Investment T ool?
Journal of Financial and Quantitative Analysis , 3(3), 315–326.
Jansen, I. P ., & Nikiforov, A. L. (2016). Fear and Greed: A Returns-Based T rading
Strategy Around Earnings Announcements. Journal of Portfolio Management, 42(4),
88–95.
Jarrow, R. A., & Protter, P . (2012). A Dysfunctional Role of High Frequency T rading in
Electronic Markets. International Journal of Theoretical and Applied Finance , 15 (3),
1250022.
Jasemi, M., & Kimiagari, A. M. (2012). An Investigation of Model Selection Cri-
teria for T echnical Analysis of Moving Average. Journal of Industrial Engineering
International, 8,5 .
Jegadeesh, N. (1990). Evidence of Predictable Behavior of Security Returns. Journal
of Finance , 45 (3), 881–898.
Jegadeesh, N., & Titman, S. (1993). Returns to Buying Winners and Selling Losers:
Implications for Stock Market Efﬁciency. Journal of Finance , 48(1), 65–91.
Jegadeesh, N., & Titman, S. (1995). Overreaction, Delayed Reaction, and Contrarian
Proﬁts. Review of Financial Studies , 8(4), 973–993.
Jegadeesh, N., & Titman, S. (2001). Proﬁtability of Momentum Strategies: An Eval-
uation of Alternative Explanations. Journal of Finance , 56 (2), 699–720.
Jensen, M. C. (1968). The Performance of Mutual Funds in the Period 1945–1964.
Journal of Finance , 23(2), 389–416.
Jetley, G., & Ji, X. (2010). The Shrinking Merger Arbitrage Spread: Reasons and
Implications. Financial Analysts Journal , 66 (2), 54–68.
Kablan, A. (2009). Adaptive Neuro-Fuzzy Inference System for Financial T rading
Using Intraday Seasonality Observation Model. International Journal of Economics
and Management Engineering , 3(10), 1909–1918.
Kahn, R. N., & Lemmon, M. (2015). Smart Beta: The Owner’s Manual. Journal of
Portfolio Management, 41(2), 76–83.
Kahn, R. N., & Lemmon, M. (2016). The Asset Manager’s Dilemma: How Smart Beta
Is Disrupting the Investment Management Industry. Financial Analysts Journal ,
72(1), 15–20.

78 Z. Kakushadze and J. A. Serur
Kahneman, D., & Tversky, A. (1979). Prospect Theory: An Analysis of Decision
Under Risk. Econometrica, 47 (2), 263–292.
Kakushadze, Z. (2015a). Mean-Reversion and Optimization. Journal of Asset Man-
agement, 16 (1), 14–40. Available online: https://ssrn.com/abstract=2478345.
Kakushadze, Z. (2015b). 4-Factor Model for Overnight Returns. Wilmott Magazine ,
2015 (79), 56–62. Available online: https://ssrn.com/abstract=2511874.
Kakushadze, Z. (2015c). On Origins of Alpha. Hedge Fund Journal , 108, 47–50.
Available online: https://ssrn.com/abstract=2575007.
Kakushadze, Z. (2015d). Heterotic Risk Models. Wilmott Magazine , 2015 (80), 40–
55. Available online: https://ssrn.com/abstract=2600798.
Kakushadze, Z. (2016). 101 Formulaic Alphas. Wilmott Magazine, 2016 (84), 72–80.
Available online: https://ssrn.com/abstract=2701346.
Kakushadze, Z., & T ulchinsky, I. (2016). Performance v. T urnover: A Story by 4,000
Alphas. Journal of Investment Strategies , 5 (2), 75–89. Available online: http://ssrn.
com/abstract=2657603.
Kakushadze, Z., & Yu, W . (2016a). Multifactor Risk Models and Heterotic CAPM.
Journal of Investment Strategies , 5 (4), 1–49. Available online: https://ssrn.com/
abstract=2722093.
Kakushadze, Z., & Yu, W . (2016b). Statistical Industry Classiﬁcation. Journal of Risk
& Control , 3(1), 17–65. Available online: https://ssrn.com/abstract=2802753.
Kakushadze, Z., & Yu, W . (2017a). Statistical Risk Models. Journal of Investment
Strategies, 6 (2), 1–40. Available online: https://ssrn.com/abstract=2732453.
Kakushadze, Z., & Yu, W . (2017b). How to Combine a Billion Alphas. Journal of Asset
Management, 18(1), 64–80. Available online: https://ssrn.com/abstract=2739219.
Kakushadze, Z., & Yu, W . (2017c). *K-Means and Cluster Models for Cancer Sig-
natures. Biomolecular Detection and Quantiﬁcation , 13, 7–31. Available online:
https://ssrn.com/abstract=2908286.
Kakushadze, Z., & Yu, W . (2018). Decoding Stock Market with Quant Alphas.
Journal of Asset Management , 19 (1), 38–48. Available online: https://ssrn.com/
abstract=2965224.
Kang, J., Liu, M. H., & Ni, S. X. (2002). Contrarian and Momentum Strategies in the
China Stock Market: 1993–2000. Paciﬁc-Basin Finance Journal , 10(3), 243–265.
Kara, Y., Boyacioglu, M. A., & Baykan, O. K. (2011). Predicting Direction of Stock
Price Index Movement Using Artiﬁcial Neural Networks and Support Vector
Machines: The Sample of the Istanbul Stock Exchange. Expert Systems with Appli-
cations, 38(5), 5311–5319.
Karolyi, G. A., & Kho, B. C. (2004). Momentum Strategies: Some Bootstrap T ests.
Journal of Empirical Finance , 11(4), 509–536.
Karolyi, G. A., & Shannon, J. (1999). Where’s the Risk in Risk Arbitrage? Canadian
Investment Review, 12(2), 12–18.
Khan, S. A. (2002). Merger Arbitrage: A Long-T erm Investment Strategy. Journal of
Wealth Management, 4 (4), 76–81.

3 Stocks 79
Khandani, A., & Lo, A. W . (2011). What Happened to the Quants in August 2007?
Evidence from Factors and T ransactions Data. Journal of Financial Markets , 14 (1),
1–46.
Kilgallen, T . (2012). T esting the Simple Moving Average Across Commodities, Global
Stock Indices, and Currencies. Journal of Wealth Management , 15 (1), 82–100.
Kim, K. (2011). Performance Analysis of Pairs T rading Strategy Utilizing High Frequency
Data with an Application to KOSPI 100 Equities (Working Paper). Available online:
https://ssrn.com/abstract=1913707.
Kim, K. J. (2003). Financial Time Series Forecasting Using Support Vector Machines.
Neurocomputing, 55 (1–2), 307–319.
Kim, K. J. (2006). Artiﬁcial Neural Networks with Evolutionary Instance Selection
for Financial Forecasting. Expert Systems with Applications , 30(3), 519–526.
Kim, K. J., & Han, I. (2000). Genetic Algorithms Approach to Feature Discretization
in Artiﬁcial Neural Networks for the Prediction of Stock Price Index. Expert Systems
with Applications , 19 (2), 125–132.
Kirilenko, A., Kyle, A., Samadi, M., & T uzun, T . (2017). The Flash Crash: High-
Frequency T rading in an Electronic Market. Journal of Finance , 72(3), 967–998.
Kishore, V . (2012).Optimizing Pairs T rading of US Equities in a High Frequency Setting
(Working Paper). Available online: https://repository.upenn.edu/cgi/viewcontent.
cgi?article=1095&context=wharton_research_scholars.
Korajczyk, R. A., & Murphy, D. (2017). High Frequency Market Making to
Large Institutional T rades (Working Paper). Available online: https://ssrn.com/
abstract=2567016.
Korajczyk, R. A., & Sadka, R. (2004). Are Momentum Proﬁts Robust to T rading
Costs? Journal of Finance , 59 (3), 1039–1082.
Kordos, M., & Cwiok, A. (2011). A New Approach to Neural Network Based Stock
T rading Strategy. In H. Yin, W . Wang, & V . Rayward-Smith (Eds.), Intelligent
Data Engineering and Automated Learning-IDEAL (pp. 429–436). Berlin, Ger-
many: Springer.
Kozhan, R., & Tham, W . W . (2012). Execution Risk in High-Frequency Arbitrage.
Management Science , 58(11), 2131–2149.
Kozlov, M., & Petajisto, A. (2013). Global Return Premiums on Earnings Quality, Value,
and Size (Working Paper). Available online: https://ssrn.com/abstract=2179247.
Krauss, C. (2017). Statistical Arbitrage Pairs T rading Strategies: Review and Outlook.
Journal of Economic Surveys , 31(2), 513–545.
Krauss, C., & Stübinger, J. (2017). Non-linear Dependence Modelling with Bivariate
Copulas: Statistical Arbitrage Pairs T rading on the S&P 100. Applied Economics ,
23(1), 1–18.
Kryzanowski, L., Galler, M., & Wright, D. (1993). Using Artiﬁcial Neural Networks
to Pick Stocks. Financial Analysts Journal , 49 (4), 21–27.
Kudryavtsev, A. (2012). Overnight Stock Price Reversals. Journal of Advanced Studies
in Finance , 3(2), 162–170.

80 Z. Kakushadze and J. A. Serur
Kumar, M., & Thenmozhi, M. (2001). Forecasting Stock Index Movement: A Com-
parison of Support Vector Machines and Random Forest (Working Paper). Available
online: https://ssrn.com/abstract=876544.
Lakonishok, J., Shleifer, A., & Vishny, R. W . (1994). Contrarian Investment, Extrap-
olation, and Risk. Journal of Finance , 49 (5), 1541–1578.
Larker, D., & Lys, T . (1987). An Empirical Analysis of the Incentives to Engage in
Costly Information Acquisition: The Case of Risk Arbitrage. Journal of Financial
Economics, 18(1), 111–126.
Lehmann, B. N. (1990). Fads, Martingales, and Market Efﬁciency. Quarterly Journal
of Economics , 105 (1), 1–28.
Li, X., Deng, X., Zhu, S., Wang, F ., & Xie, H. (2014). An Intelligent Market Making
Strategy in Algorithmic T rading. Frontiers of Computer Science , 8(4), 596–608.
Li, B., Hoi, S. C. H., Sahoo, D., & Liu, Z.-Y. (2015). Moving Average Reversion
Strategy for On-line Portfolio Selection. Artiﬁcial Intelligence , 222, 104–123.
Li, X., Sullivan, R. N., & Garcia-Feijóo, L. (2014). The Limits to Arbitrage and the
Low-Volatility Anomaly. Financial Analysts Journal , 70(1), 52–63.
Li, X., Sullivan, R. N., & Garcia-Feijóo, L. (2016). The Low-Volatility Anomaly:
Market Evidence on Systematic Risk vs. Mispricing. Financial Analysts Journal ,
72(1), 36–47.
Li, B., Zhao, P ., Hoi, S. C. H., & Gopalkrishnan, V . (2012). PAMR: Passive Aggressive
Mean Reversion Strategy for Portfolio Selection. Machine Learning , 87 (2), 221–
258.
Liew, J. K.-S., & Mayster, B. (2018). Forecasting ETFs with Machine Learning Algo-
rithms. Journal of Alternative Investments , 20(3), 58–78.
Liew, J., & Roberts, R. (2013). U.S. Equity Mean-Reversion Examined. Risks, 1(3),
162–175.
Liew, J., & Vassalou, M. (2000). Can Book-to-Market, Size and Momentum be Risk
Factors that Predict Economic Growth? Journal of Financial Economics , 57 (2),
221–245.
Liew, R., & Wu, Y. (2013). Pairs T rading: A Copula Approach. Journal of Derivatives
& Hedge Funds , 19 (1), 12–30.
Lin, L., Lan, L.-H., & Chuang, S.-S. (2013). An Option-Based Approach to Risk
Arbitrage in Emerging Markets: Evidence from T aiwan T akeover Attempts. Journal
of Forecasting, 32(6), 512–521.
Lin, Y.-X., McCrae, M., & Gulati, C. (2006). Loss Protection in PairsT radingThrough
Minimum Proﬁt Bounds: A Cointegration Approach. Journal of Applied Mathe-
matics and Decision Sciences , 4, 1–14.
Liu, B., Chang, L. B., & Geman, H. (2017). Intraday Pairs T rading Strategies on
High Frequency Data: The Case of Oil Companies. Quantitative Finance , 17 (1),
87–100.
Liu, L. X., & Zhang, L. (2008). Momentum Proﬁts, Factor Pricing, and Macroeco-
nomic Risk. Review of Financial Studies , 21(6), 2417–2448.

3 Stocks 81
Livnat, J., & Mendenhall, R. R. (2006). Comparing the Post-Earnings Announcement
Drift for Surprises Calculated from Analyst and Time Series Forecasts. Journal of
Accounting Research, 44 (1), 177–205.
Lo, A. W . (2008). Where Do Alphas Come From? A New Measure of the Value of
Active Investment Management. Journal of Investment Management , 6 (2), 1–29.
Lo, A. W ., & MacKinlay, A. C. (1990). When Are Contrarian Proﬁts Due to Stock
Market Overreaction? Review of Financial Studies , 3(3), 175–205.
Lo, A. W ., Mamaysky, H., & Wang, J. (2000). Foundations of T echnical Analysis:
Computational Algorithms, Statistical Inference, and Empirical Implementation.
Journal of Finance , 55 (4), 1705–1765.
Loh, R. K., & Warachka, M. (2012). Streaks in Earnings Surprises and the Cross-
Section of Stock Returns. Management Science , 58(7), 1305–1321.
Lu, C. J., Lee, T . S., & Chiu, C. (2009). Financial Time Series Forecasting Using
Independent Component Analysis and Support Vector Regression. Decision Support
Systems, 47 (2), 115–125.
Madhavan, A. (2012). Exchange-T raded Funds, Market Structure, and the Flash
Crash. Financial Analysts Journal , 68(4), 20–35.
Maheswaran, K., & Yeoh, S. C. (2005). The Proﬁtability of Merger Arbitrage: Some
Australian Evidence. Australian Journal of Management , 30(1), 111–126.
Malkiel, B. G. (2014). Is Smart Beta Really Smart? Journal of Portfolio Management ,
40(5), 127–134.
Markowitz, H. (1952). Portfolio Selection. Journal of Finance , 7 (1), 77–91.
Mendenhall, R. (2004). Arbitrage Risk and the Post-Earnings-Announcement Drift.
Journal of Business , 77 (6), 875–894.
Menkveld, A. J. (2013). High Frequency T rading and the New Market Makers. Journal
of Financial Markets , 16 (4), 712–740.
Menkveld, A. J. (2016). The Economics of High-Frequency T rading: T aking Stock.
Annual Review of Financial Economics , 8, 1–24.
Merton, R. C. (1987). A Simple Model of Capital Market Equilibrium with Incom-
plete Information. Journal of Finance , 42(3), 483–510.
Metghalchi, M., Marcucci, J., & Chang, Y.-H. (2012). Are Moving Average T rading
Rules Proﬁtable? Evidence from the European Stock Markets. Applied Economics ,
44 (12), 1539–1559.
Miao, G. J. (2014). High Frequency and Dynamic Pairs T rading Based on Statistical
Arbitrage Using a T wo-Stage Correlation and Cointegration Approach. Interna-
tional Journal of Economics and Finance , 6 (3), 96–110.
Milosevic, N. (2016). Equity Forecast: Predicting Long T erm Stock Price Movement
Using Machine Learning. Journal of Economics Library , 3(2), 288–294.
Mitchell, M., & Pulvino, T . (2001). Characteristics of Risk and Return in Risk Arbi-
trage. Journal of Finance , 56 (6), 2135–2175.
Moskowitz, T . J., & Grinblatt, M. (1999). Do Industries Explain Momentum? Journal
of Finance , 54 (4), 1249–1290.

82 Z. Kakushadze and J. A. Serur
Mun, J. C., Vasconcellos, G. M., & Kish, R. (2000). The Contrarian Overreaction
Hypothesis: An Analysis of the US and Canadian Stock Markets. Global Finance
Journal, 11(1–2), 53–72.
Murphy, J. J. (1986). T echnical Analysis of the Futures Markets: A Comprehensive Guide
to T rading Methods and Applications. New York, NY: New York Institute of Finance.
Muthuswamy, J., Palmer, J., Richie, N., & Webb, R. (2011). High-Frequency T rading:
Implications for Markets, Regulators, and Efﬁciency. Journal of T rading, 6 (1), 87–
97.
Ng, J., Rusticus, T ., & Verdi, R. (2008). Implications of T ransaction Costs for the
Post-Earnings Announcement Drift. Journal of Accounting Research , 46 (3), 661–
696.
Novak, M. G., & Velušçek, D. (2016). Prediction of Stock Price Movement Based on
Daily High Prices. Quantitative Finance , 16 (5), 793–826.
Novy-Marx, R. (2013). The Other Side of Value: The Gross Proﬁtability Premium.
Journal of Financial Economics , 108(1), 1–28.
Ofﬁcer, M. S. (2004). Collars and Renegotiation in Mergers and Acquisitions. Journal
of Finance , 59 (6), 2719–2743.
Ofﬁcer, M. S. (2006). The Market Pricing of Implicit Options in Merger Collars.
Journal of Business , 79 (1), 115–136.
O’Hara, M. (2015). High Frequency Market Microstructure. Journal of Financial
Economics, 116 (2), 257–270.
Osler, C. L. (2000). Support for Resistance: T echnical Analysis and Intraday Exchange
Rates. Federal Reserve Bank of New York, Economic Policy Review , 6 (2), 53–68.
Osler, C. L. (2003). Currency Orders and Exchange Rate Dynamics: An Explanation
for the Predictive Success of T echnical Analysis. Journal of Finance , 58(5), 1791–
1819.
O’T ool, R. (2013).The Black-Litterman Model: A Risk Budgeting Perspective.Journal
of Asset Management , 14 (1), 2–13.
Ou, P ., & Wang, H. (2009). Prediction of Stock Market Index Movement by T en
Data Mining T echniques. Modern Applied Science , 3(12), 28–42.
Pagnotta, E., & Philippon, T . (2012). Competing on Speed (Working Paper). Available
online: https://ssrn.com/abstract=1972807.
Pan, J., & Poteshman, A. M. (2006). The Information in Option Volume for Future
Stock Prices. Review of Financial Studies , 19 (3), 871–908.
Pástor, L ’., & Stambaugh, R. F . (2003). Liquidity Risk and Expected Stock Returns.
Journal of Political Economy , 111(3), 642–685.
Pätäri, E., & Vilska, M. (2014). Performance of Moving Average T rading Strategies
Over Varying Stock Market Conditions: The Finnish Evidence. Applied Economics,
46 (24), 2851–2872.
Perlin, M. S. (2009). Evaluation of Pairs-T rading Strategy at the Brazilian Financial
Market. Journal of Derivatives & Hedge Funds , 15 (2), 122–136.
Person, J. L. (2007). Candlestick and Pivot Point T rading T riggers. Hoboken, NJ: Wiley.

3 Stocks 83
Piotroski, J. D. (2000). Value Investing: The Use of Historical Financial Statement
Information to Separate Winners from Losers. Journal of Accounting Research , 38,
1–41.
Piotroski, J. D., & So, E. C. (2012). Identifying Expectation Errors in Value/Glamour
Strategies: A Fundamental Analysis Approach. Review of Financial Studies , 25 (9),
2841–2875.
Pizzutilo, F . (2013). A Note on the Effectiveness of Pairs T rading for Individual
Investors. International Journal of Economics and Financial Issues , 3(3), 763–771.
Pole, A. (2007). Statistical Arbitrage: AlgorithmicT rading Insights andT echniques. Hobo-
ken, NJ: Wiley.
Poterba, J. M., & Summers, L. H. (1988). Mean Reversion in Stock Prices: Evidence
and Implications. Journal of Financial Economics , 22(1), 27–59.
Pring, M. J. (1985). T echnical Analysis Explained: The Successful Investor’s Guide to
Spotting Investment T rends and T urning Points (3rd ed.). New York, NY: McGraw-
Hill Inc.
Rad, H., Low, R. K. Y., & Faff, R. (2016). The Proﬁtability of Pairs T rading Strate-
gies: Distance, Cointegration and Copula Methods. Quantitative Finance, 16 (10),
1541–1558.
Refenes, A. N., Zapranis, A. S., & Francis, G. (1994). Stock Performance Model-
ing Using Neural Networks: Comparative Study with Regressive Models. Neural
Networks, 7 (2), 375–388.
Rendleman, R. J., Jones, C. P ., & Latané, H. A. (1982). Empirical Anomalies Based on
Unexpected Earnings and the Importance of Risk Adjustments. Journal of Financial
Economics, 10(3), 269–287.
Riordan, R., & Storkenmaier, A. (2012). Latency, Liquidity and Price Discovery.
Journal of Financial Markets , 15 (4), 416–437.
Rodríguez-González, A., García-Crespo, Á., Colomo-Palacios, R., Iglesias, F . G., &
Gómez-Berbís, J. M. (2011). CAST: Using Neural Networks to Improve T rading
Systems Based on T echnical Analysis by Means of the RSI Financial Indicator.
Expert Systems with Applications , 38(9), 11489–11500.
Rosenberg, B., Reid, K., & Lanstein, R. (1985). Persuasive Evidence of Market Inef-
ﬁciency. Journal of Portfolio Management , 11(3), 9–16.
Rouwenhorst, K. G. (1998). International Momentum Strategies. Journal of Finance ,
53(1), 267–284.
Saad, E. W ., Prokhorov, D. V ., & Wunsch, D. C. (1998). Comparative Study of Stock
T rend Prediction Using Time Delay, Recurrent and Probabilistic Neural Networks.
IEEE T ransactions on Neural Networks , 9 (6), 1456–1470.
Sadka, R. (2002). The Seasonality of Momentum: Analysis of T radability (Working
Paper). Available online: https://ssrn.com/abstract=306371.
Samuelson, W ., & Rosenthal, L. (1986). Price Movements as Indicators of T ender
Offer Success. Journal of Finance , 41(2), 481–499.
Samworth, R. J. (2012). Optimal Weighted Nearest Neighbour Classiﬁers. Annals of
Statistics, 40
(5), 2733–2763.

84 Z. Kakushadze and J. A. Serur
Satchell, S., & Scowcroft, A. (2000). A Demystiﬁcation of the Black-Litterman Model:
Managing Quantitative and T raditional Portfolio Construction. Journal of Asset
Management, 1(2), 138–150.
Schiereck, D., Bondt, W . D., & Weber, M. (1999). Contrarian and Momentum
Strategies in Germany. Financial Analysts Journal , 55 (6), 104–116.
Scholes, M., & Williams, J. (1977). Estimating Betas from Nonsynchronous Data.
Journal of Financial Economics , 5 (3), 309–327.
Schumaker, R. P ., & Chen, H. (2010). A Discrete Stock Price Prediction Engine Based
on Financial News. Computer, 43(1), 51–56.
Sharpe, W . F . (1966). Mutual Fund Performance.Journal of Business , 39 (1), 119–138.
Sharpe, W . F . (1994). The Sharpe Ratio. Journal of Portfolio Management , 21(1), 49–
58.
Shi, H.-L., Jiang, Z.-Q., & Zhou, W .-X. (2015). Proﬁtability of Contrarian Strategies
in the Chinese Stock Market. PLoS ONE , 10(9), e0137892.
Shiu, Y.-M., & Lu, T .-H. (2011). Pinpoint and Synergistic T rading Strategies of Can-
dlesticks. International Journal of Economics and Finance , 3(1), 234–244.
Siganos, A., & Chelley-Steeley, P . (2006). Momentum Proﬁts Following Bull and Bear
Markets. Journal of Asset Management , 6 (5), 381–388.
Stattman, D. (1980). Book Values and Stock Returns. Chicago MBA: A Journal of
Selected Papers, 4, 25–45.
Stickel, S. E. (1991). Common Stock Returns Surrounding Earnings Forecast Revi-
sions: More Puzzling Evidence. Accounting Review , 66 (2), 402–416.
Stivers, C., & Sun, L. (2010). Cross-Sectional Return Dispersion and Time Variation
in Value and Momentum Premiums. Journal of Financial and Quantitative Analysis ,
45 (4), 987–1014.
Stübinger, J., & Bredthauer, J. (2017). Statistical Arbitrage Pairs T rading with High-
Frequency Data. International Journal of Economics and Financial Issues , 7 (4), 650–
662.
Stübinger, J., & Endres, S. (2017). Pairs T rading with a Mean-Reverting Jump-
Diffusion Model on High-Frequency Data. Quantitative Finance (forthcoming).
https://doi.org/10.1080/14697688.2017.1417624.
Subha, M., & Nambi, S. (2012). Classiﬁcation of Stock Index Movement Using k-
Nearest Neighbours (k-NN) Algorithm. WSEAS T ransactions on Information Science
and Applications , 9 (9), 261–270.
Subramanian, A. (2004). Option Pricing on Stocks in Mergers and Acquisitions.
Journal of Finance , 59 (2), 795–829.
Suhonen, A., Lennkh, M., & Perez, F . (2017). Quantifying Backtest Overﬁtting in
Alternative Beta Strategies. Journal of Portfolio Management , 43(2), 90–104.
Sullivan, R., Timmermann, A., & White, H. (1999). Data-Snooping, T echnical T rad-
ing Rule Performance, and the Bootstrap. Journal of Finance , 54 (5), 1647–1691.
T ay, F . E. H., & Cao, L. (2001). Application of Support Vector Machines in Financial
Time Series Forecasting. Omega, 29 (4), 309–317.
T aylor, M. P ., & Allen, H. (1992). The Use of T echnical Analysis in the Foreign
Exchange Market. Journal of International Money and Finance , 11(3), 304–314.

3 Stocks 85
T eixeira, L. A., & de Oliveira, A. L. I. (2010). A Method for Automatic Stock T rading
Combining T echnical Analysis and Nearest Neighbor Classiﬁcation. Expert Systems
with Applications , 37 (10), 6885–6890.
Thomsett, M. C. (2003). Support and Resistance Simpliﬁed . Columbia, MD: Market-
place Books.
Tsai, C. F ., & Hsiao, Y. C. (2010). Combining Multiple Feature Selection Methods for
Stock Prediction: Union, Intersection, and Multi-intersection Approaches. Decision
Support Systems , 50(1), 258–269.
T ulchinsky, I., et al. (2015). Finding Alphas: A Quantitative Approach to Building
T rading Strategies.N e wY o r k ,N Y :W i l e y .
Vaitonis, M., & Masteika, S. (2016). Research in High Frequency T rading and Pairs
Selection Algorithm with Baltic Region Stocks. In G. Dregvaite & R. Damasevicius
(Eds.), Proceedings of the 22nd International Conference on Information and Software
T echnologies (ICIST 2016) (pp. 208–217). Cham, Switzerland: Springer.
Van Kervel, V ., & Menkveld, A. J. (2017). High-Frequency T rading Around Large
Institutional Orders. Journal of Finance (forthcoming). Available online: https://
ssrn.com/abstract=2619686.
Van Oord, J. A. (2016). Essays on Momentum Strategies in Finance . Ph.D. thesis,
Erasmus University, Rotterdam, The Netherlands. Available online: https://repub.
eur.nl/pub/80036/EPS2016380F-A9789058924445.pdf .
Van T assel, P . (2016). Merger Options and Risk Arbitrage (Federal Reserve Bank of
New York Staff Reports, No. 761). Available online: https://www.newyorkfed.org/
medialibrary/media/research/staff_reports/sr761.pdf?la=en.
Vanstone, B., & Finnie, G. (2009). An Empirical Methodology for Developing Stock-
market T rading Systems Using Artiﬁcial Neural Networks. Expert Systems with
Applications, 36 (3), 6668–6680.
Vidyamurthy, G. (2004). Pairs T rading: Quantitative Methods and Analysis . Hoboken,
NJ: Wiley.
Walkling, R. A. (1985). Predicting T ender Offer Success: A Logistic Analysis. Journal
of Financial and Quantitative Analysis , 20(4), 461–478.
Wang, K. Q. (2005). Multifactor Evaluation of Style Rotation. Journal of Financial
and Quantitative Analysis , 40(2), 349–372.
Watts, R. L. (1978). Systematic ‘ Abnormal’ Returns After Quarterly Earnings
Announcements. Journal of Financial Economics , 6 (2–3), 127–150.
Weller, P . A., Friesen, G. C., & Dunham, L. M. (2009). Price T rends and Patterns in
T echnical Analysis: A Theoretical and Empirical Examination. Journal of Banking
& Finance , 6 (33), 1089–1100.
Xie, W ., Liew, Q. R., Wu, Y., & Zou, X. (2014). Pairs T rading with Copulas (Working
Paper). Available online: https://ssrn.com/abstract=2383185.
Xing, Y., Zhang, X., & Zhao, R. (2010). What Does Individual Option Volatility
Smirk T ell Us About Future Equity Returns? Journal of Financial and Quantitative
Analysis, 45 (3), 641–662.
Yao, Y. (2012). Momentum, Contrarian, and the January Seasonality. Journal of Bank-
ing & Finance ,
36 (10), 2757–2769.

86 Z. Kakushadze and J. A. Serur
Yao, J., & T an, C. L. (2000). A Case Study on Using Neural Networks to Perform
T echnical Forecasting of Forex. Neurocomputing, 34 (1–4), 79–98.
Yao, J., T an, C. L., & Poh, H. L. (1999). Neural Networks for T echnical Analysis:
A Study on KLCI. International Journal of Theoretical and Applied Finance , 2(2),
221–241.
Yoshikawa, D. (2017). An Entropic Approach for Pair T rading. Entropy, 19 (7), 320.
Yu, L., Wang, S., & Lai, K. K. (2005). Mining Stock Market T endency Using GA-
Based Support Vector Machines. In X. Deng & Y. Ye (Eds.), Internet and Network
Economics. WINE 2005. Lecture Notes in Computer Science (Vol. 3828, pp. 336–
345). Berlin, Germany: Springer.
Zakamulin, V . (2014). The Real-Life Performance of Market Timing with Moving
Average and Time-Series Momentum Rules. Journal of Asset Management , 15 (4),
261–278.
Zakamulin, V . (2015). A Comprehensive Look at the Empirical Performance of Mov-
ing Average T rading Strategies (Working Paper). Available online: https://ssrn.com/
abstract=2677212.
Zapranis, A., & Tsinaslanidis, P . E. (2012). Identifying and Evaluating Horizontal
Support and Resistance Levels: An Empirical Study on US Stock Markets. Applied
Financial Economics , 22(19), 1571–1585.
Zeng, Z., & Lee, C. G. (2014). Pairs T rading: Optimal Thresholds and Proﬁtability.
Quantitative Finance , 14 (11), 1881–1893.
Zhang, L. (2005). The Value Premium. Journal of Finance , 60(1), 67–103.
# Chapter 2: Options

2
Options
2.1 Generalities
An option is a form of a ﬁnancial derivative. It is a contract sold by the option
writer to the option holder. T ypically, an option gives the option holder the
right, but not the obligation, to buy or sell an underlying security or ﬁnancial
asset (e.g., a share of common stock) at an agreed-upon price (referred to as
the strike price) during a certain period of time or on a speciﬁc date (referred
to as the exercise date). A buyer pays a premium to the seller for the option.
For option pricing, see, e.g., Harrison and Pliska ( 1981), Baxter and Rennie
(1996), Hull ( 2012), Kakushadze ( 2015).
A European call option is a right (but not an obligation) to buy a stock at
the maturity time T for the strike price k agreed on at time t = 0. The claim
for the call option f
call (ST , k) = (ST − k)+.H e r e (x )+ = x if x > 0,a n d
(x )+ = 0 if x ≤ 0. By the “claim” we mean how much the option is worth at
maturity T . If the stock price at maturity ST > k, then the option holder gains
ST −k (excluding the cost paid for the option at t = 0). If the price at maturity
ST ≤ k, then there is no proﬁt to be made from the option as it makes no
sense to exercise it if ST < k (as it is cheaper to buy the stock in the market)
and it makes no difference if ST = k—all this is assuming no transaction
costs. Similarly, a European put option is a right (but not an obligation) to sell
a stock at the maturity time T for the strike price k agreed on at time t = 0.
The claim for the put option is given by f put (ST , k) = (k − ST )+.
Options can be issued on a variety of underlying assets, e.g., equities (single-
stock options), bonds, futures, indexes, commodities, currencies, etc. For the
sake of terminological convenience and deﬁniteness, in the following we will
© The Author(s) 2018
Z. Kakushadze and J. A. Serur, 151 T rading Strategies,
https://doi.org/10.1007/978-3-030-02792-6_2
5

6 Z. Kakushadze and J. A. Serur
frequently refer to the underlying asset as “stock”, even though in many cases
the discussion can be readily generalized to other assets. Furthermore, there is
a variety of option styles (beyond European options—for European options,
see, e.g., Black and Scholes [ 1973]), e.g., American options (that can be exer-
cised on any trading day on or before expiration—see, e.g., Kim [ 1990]),
Bermudan options (that can be exercised only on speciﬁed dates on or before
expiration—see, e.g., Andersen [ 1999]), Canary options (that can be exer-
cised, say, quarterly, but not before a determined time period, say, 1 year, has
elapsed—see, e.g., Henrard [ 2006]), Asian options (whose payoff is deter-
mined by the average underlying price over some preset time period—see,
e.g., Rogers and Shi [ 1995]), barrier options (which can be exercised only
if the underlying security’s price passes a certain level or “barrier”—see, e.g.,
Haug [2001]), other exotic options (a broad category of options that typically
are complexly structured—see, e.g., Fabozzi [ 2002]), etc. Let us also mention
binary (a.k.a. all-or-nothing or digital) options (that pay a preset amount, say,
$1, if the underlying security meets a predeﬁned condition on expiration, oth-
erwise they simply expire without paying anything to the holder—see, e.g.,
Breeden and Litzenberger [ 1978]).
Some trading strategies can be built using, e.g., combinations of options.
Such trading strategies can be divided into two groups: directional and non-
directional. Directional strategies imply an expectation on the direction of the
future stock price movements. Non-directional (a.k.a. neutral) strategies are
not based on the future direction: the trader is oblivious to whether the stock
price goes up or down.
Directional strategies can be divided into two subgroups: (i) bullish strate-
gies, where the trader proﬁts if the stock price goes up; and (ii) bearish strategies,
where the trader proﬁts if the stock price goes down. Non-directional strate-
gies can be divided into two subgroups: (a) volatility strategies that proﬁt
if the stock has large price movements (high volatility environment); and
(b) sideways strategies that proﬁt if the stock price remains stable (low volatility
environment). Also, one can distinguish income, capital gain, hedging strate-
gies, etc. (see, e.g., Cohen 2005).
In the remainder of this section, unless stated otherwise, all options are for
the same stock and have the same time-to-maturity (TTM). The moneyness
abbreviations are: ATM = at-the-money, ITM = in-the-money, OTM = out-
of-the-money. Also: f
T is the payoff at maturity T ; S0 is the stock price at
the time t = 0 of entering the trade (i.e., establishing the initial position); ST
is the stock price at maturity; C is the net credit received at t = 0,a n d D is
the net debit required at t = 0, as applicable; H = D (for a net debit trade)

2 Options 7
or H =− C (for a net credit trade) 1; S∗up and S∗down are the higher and
lower break-even (i.e., for which fT = 0) stock prices at maturity; if there is
only one break-even price, it is denoted by S∗; Pmax is the maximum proﬁt at
maturity; Lmax is the maximum loss at maturity.
2.2 Strategy: Covered Call
This strategy (a.k.a. “buy-write” strategy) amounts to buying stock and writing
a call option with a strike price K against the stock position. The trader’s
outlook on the stock price is neutral to bullish. The covered call strategy has
the same payoff as writing a put option (short/naked put). 2 While maintaining
the long stock position, the trader can generate income by periodically selling
OTM call options. We have 3:
fT = ST − S0 − (ST − K )+ + C = K − S0 − (K − ST )+ + C (2.1)
S∗ = S0 − C (2.2)
Pmax = K − S0 + C (2.3)
Lmax = S0 − C (2.4)
2.3 Strategy: Covered Put
This strategy (a.k.a. “sell-write” strategy) amounts to shorting stock and writing
a put option with a strike price K against the stock position. The trader’s
outlook is neutral to bearish. The covered put strategy has the same payoff
as writing a call option (short/naked call). While maintaining the short stock
position, the trader can generate income by periodically selling OTM put
options. We have
4:
fT = S0 − ST − (K − ST )+ + C = S0 − K − (ST − K )+ + C (2.5)
S∗ = S0 + C (2.6)
1 H is the net debit for all bought option premia less the net credit for all sold option premia.
2This is related to put-call parity (see, e.g., Stoll 1969;H u l l 2012).
3For some literature on covered call strategies, see, e.g., Pounds ( 1978), Whaley ( 2002), Feldman and Roy
(2004), Hill et al. ( 2006), Kapadia and Szado ( 2007), Che and Fung ( 2011), Mugwagwa et al. ( 2012),
Israelov and Nielsen ( 2014, 2015a), Hemler and Miller ( 2015).
4The covered put option strategy is symmetrical to the covered call option strategy. Academic literature
on the covered put option strategy appears to be scarce. See, e.g., Che ( 2016).

8 Z. Kakushadze and J. A. Serur
Pmax = S0 − K + C (2.7)
Lmax = unlimited (2.8)
2.4 Strategy: Protective Put
This strategy (a.k.a. “married put” or “synthetic call”) amounts to buying stock
and an ATM or OTM put option with a strike price K ≤ S0.T h et r a d e r ’ s
outlook is bullish. This is a hedging strategy: the put option hedges the risk of
the stock price falling. We have
5:
fT = ST − S0 + (K − ST )+ − D = K − S0 + (ST − K )+ − D (2.9)
S∗ = S0 + D (2.10)
Pmax = unlimited (2.11)
Lmax = S0 − K + D (2.12)
2.5 Strategy: Protective Call
This strategy (a.k.a. “married call” or “synthetic put”) amounts to shorting
stock and buying an ATM or OTM call option with a strike price K ≥ S0.
The trader’s outlook is bearish. This is a hedging strategy: the call option hedges
the risk of the stock price rising. We have 6:
fT = S0 − ST + (ST − K )+ − D = S0 − K + (K − ST )+ − D (2.13)
S∗ = S0 − D (2.14)
Pmax = S0 − D (2.15)
Lmax = K − S0 + D (2.16)
2.6 Strategy: Bull Call Spread
This is a vertical spread consisting of a long position in a close to ATM call
option with a strike price K1, and a short position in an OTM call option
5For some literature on protective put strategies, see, e.g., Figlewski et al. ( 1993), Israelov and Nielsen
(2015b), Israelov et al. ( 2017), Israelov ( 2017).
6The protective call option strategy is symmetrical to the protective put option strategy. Academic literature
on the protective call option strategy appears to be scarce. See, e.g., Jabbour and Budwick ( 2010), T okic
(2013).

2 Options 9
with a higher strike price K2. This is a net debit trade. The trader’s outlook is
bullish: the strategy proﬁts if the stock price rises. This is a capital gain strategy.
We have7:
fT = (ST − K1)+ − (ST − K2)+ − D (2.17)
S∗ = K1 + D (2.18)
Pmax = K2 − K1 − D (2.19)
Lmax = D (2.20)
2.7 Strategy: Bull Put Spread
This is a vertical spread consisting of a long position in an OTM put option
with a strike price K1, and a short position in another OTM put option with a
higher strike price K2. This is a net credit trade. The trader’s outlook is bullish.
This is an income strategy. We have:
fT = (K1 − ST )+ − (K2 − ST )+ + C (2.21)
S∗ = K2 − C (2.22)
Pmax = C (2.23)
Lmax = K2 − K1 − C (2.24)
2.8 Strategy: Bear Call Spread
This is a vertical spread consisting of a long position in an OTM call option
with a strike price K1, and a short position in another OTM call option with a
lower strike price K2. This is a net credit trade. The trader’s outlook is bearish.
This is an income strategy. We have:
fT = (ST − K1)+ − (ST − K2)+ + C (2.25)
S∗ = K2 + C (2.26)
Pmax = C (2.27)
Lmax = K1 − K2 − C (2.28)
7For some literature on bull/bear call/put vertical spreads, see, e.g., Cartea and Pedraz ( 2012), Chaput
and Ederington ( 2003, 2005), Chen et al. ( 1999), Cong et al. ( 2013, 2014), Matsypura and Timkovsky
(2010), Shah ( 2017), Wong et al. ( 2011), Zhang ( 2015). Also see Clarke et al. ( 2013), Cohen ( 2005),
Jabbour and Budwick ( 2010), McMillan ( 2002), The Options Institute ( 1995).

10 Z. Kakushadze and J. A. Serur
2.9 Strategy: Bear Put Spread
This is a vertical spread consisting of a long position in a close to ATM put
option with a strike price K1, and a short position in an OTM put option
with a lower strike price K2. This is a net debit trade. The trader’s outlook is
bearish: this strategy proﬁts if the stock price falls. This is a capital gain strategy.
We have:
fT = (K1 − ST )+ − (K2 − ST )+ − D (2.29)
S∗ = K1 − D (2.30)
Pmax = K1 − K2 − D (2.31)
Lmax = D (2.32)
2.10 Strategy: Long Synthetic Forward
This strategy amounts to buying an ATM call option and selling an ATM put
option with a strike price K = S0. This can be a net debit or net credit trade.
T ypically,|H |≪ S0. The trader’s outlook is bullish: this strategy mimics a
long stock or futures position; it replicates a long forward contract with the
delivery price K and the same maturity as the options. This is a capital gain
strategy. We have 8:
fT = (ST − K )+ − (K − ST )+ − H = ST − K − H (2.33)
S∗ = K + H (2.34)
Pmax = unlimited (2.35)
Lmax = K + H (2.36)
2.11 Strategy: Short Synthetic Forward
This strategy amounts to buying an ATM put option and selling an ATM call
option with a strike price K = S0. This can be a net debit or net credit trade.
T ypically,|H |≪ S0. The trader’s outlook is bearish: this strategy mimics a
short stock or futures position; it replicates a short forward contract with the
8For some literature on long/short synthetic forward contracts (a.k.a. synthetic futures), see, e.g., Benavides
(2009), Bozic and Fortenbery ( 2012), DeMaskey ( 1995), Ebrahim and Rahman ( 2005), Nandy and
Chattopadhyay ( 2016).

2 Options 11
delivery price K and the same maturity as the options. This is a capital gain
strategy. We have:
fT = (K − ST )+ − (ST − K )+ − H = K − ST − H (2.37)
S∗ = K − H (2.38)
Pmax = K − H (2.39)
Lmax = unlimited (2.40)
2.12 Strategy: Long Combo
This strategy (a.k.a. “long risk reversal”) amounts to buying an OTM call
option with a strike price K1 and selling an OTM put option with a strike
price K2. The trader’s outlook is bullish. This is a capital gain strategy. 9
We have ( K1 > K2):
fT = (ST − K1)+ − (K2 − ST )+ − H (2.41)
S∗ = K1 + H, H > 0 (2.42)
S∗ = K2 + H, H < 0 (2.43)
K2 ≤ S∗ ≤ K1, H = 0 (2.44)
Pmax = unlimited (2.45)
Lmax = K2 + H (2.46)
2.13 Strategy: Short Combo
This strategy (a.k.a. “short risk reversal”) amounts to buying an OTM put
option with a strike price K1 and selling an OTM call option with a strike
price K2. The trader’s outlook is bearish. This is a capital gain strategy.
We have ( K2 > K1):
fT = (K1 − ST )+ − (ST − K2)+ − H (2.47)
S∗ = K1 − H, H > 0 (2.48)
S∗ = K2 − H, H < 0 (2.49)
K1 ≤ S∗ ≤ K2, H = 0 (2.50)
9For some literature on long/short combo strategies, see, e.g., Rusnáková et al. ( 2015), Šoltés ( 2011),
Šoltés and Rusnáková ( 2012). Also see, e.g., Chaput and Ederington ( 2003).

12 Z. Kakushadze and J. A. Serur
Pmax = K1 − H (2.51)
Lmax = unlimited (2.52)
2.14 Strategy: Bull Call Ladder
This is a vertical spread consisting of a long position in (usually) a close to ATM
call option with a strike price K1, a short position in an OTM call option with a
strike price K2, and a short position in another OTM call option with a higher
strike price K3. A bull call ladder is a bull call spread ﬁnanced by selling another
OTM call option (with the strike price K3).10 This adjusts the trader’s outlook
from bullish (bull call spread) to conservatively bullish or even non-directional
(with an expectation of low volatility). We have:
fT = (ST − K1)+ − (ST − K2)+ − (ST − K3)+ − H (2.53)
S∗down = K1 + H, H > 0 (2.54)
S∗up = K3 + K2 − K1 − H (2.55)
Pmax = K2 − K1 − H (2.56)
Lmax = unlimited (2.57)
2.15 Strategy: Bull Put Ladder
This is a vertical spread consisting of a short position in (usually) a close to
ATM put option with a strike price K1, a long position in an OTM put option
with a strike price K2, and a long position in another OTM put option with a
lower strike price K3. A bull put ladder typically arises when a bull put spread (a
bullish strategy) goes wrong (the stock trades lower), so the trader buys another
OTM put option (with the strike price K
3) to adjust the position to bearish.
We have11:
fT = (K3 − ST )+ + (K2 − ST )+ − (K1 − ST )+ − H (2.58)
S∗up = K1 + H, H < 0 (2.59)
S∗down = K3 + K2 − K1 − H (2.60)
10In this sense, this is an “income” strategy.
11For some literature on ladder strategies, see, e.g., Amaitiek et al. ( 2010), Harˇcariková and Šoltés ( 2016),
He et al. ( 2016), Šoltés and Amaitiek ( 2010a).

2 Options 13
Pmax = K3 + K2 − K1 − H (2.61)
Lmax = K1 − K2 + H (2.62)
2.16 Strategy: Bear Call Ladder
This is a vertical spread consisting of a short position in (usually) a close to ATM
call option with a strike price K1, a long position in an OTM call option with a
strike price K2, and a long position in another OTM call option with a higher
strike price K3. A bear call ladder typically arises when a bear call spread (a
bearish strategy) goes wrong (the stock trades higher), so the trader buys another
OTM call option (with the strike price K3) to adjust the position to bullish.
We have:
fT = (ST − K3)+ + (ST − K2)+ − (ST − K1)+ − H (2.63)
S∗down = K1 − H, H < 0 (2.64)
S∗up = K3 + K2 − K1 + H (2.65)
Pmax = unlimited (2.66)
Lmax = K2 − K1 + H (2.67)
2.17 Strategy: Bear Put Ladder
This is a vertical spread consisting of a long position in (usually) a close to ATM
put option with a strike price K1, a short position in an OTM put option with
as t r i k ep r i c eK2, and a short position in another OTM put option with a
lower strike price K3. A bear put ladder is a bear put spread ﬁnanced by
selling another OTM put option (with the strike price K3).12 This adjusts
the trader’s outlook from bearish (bear put spread) to conservatively bearish or
even non-directional (with an expectation of low volatility). We have (assuming
K3 + K2 − K1 + H > max(H, 0)):
fT = (K1 − ST )+ − (K2 − ST )+ − (K3 − ST )+ − H (2.68)
S∗up = K1 − H, H > 0 (2.69)
S∗down = K3 + K2 − K1 + H (2.70)
12In this sense, as for the bull call ladder, this is an “income” strategy.

14 Z. Kakushadze and J. A. Serur
Pmax = K1 − K2 − H (2.71)
Lmax = K3 + K2 − K1 + H (2.72)
2.18 Strategy: Calendar Call Spread
This is a horizontal spread consisting of a long position in a close to ATM
call option with TTM T ′ and a short position in another call option with the
same strike price K but shorter TTM T < T ′. This is a net debit trade. The
trader’s outlook is neutral to bullish. At the expiration of the short call option
(t = T ), the best case scenario is if the stock price is right at the strike price
(ST = K ). At t = T let V be the value of the long call option (expiring at
t = T ′) assuming ST = K .W eh a v e13:
Pmax = V − D (2.73)
Lmax = D (2.74)
If at the expiration of the short call option the stock price Sstop −loss ≤ ST ≤
K ,w h e r eSstop −loss is the stop-loss price below which the trader would unwind
the entire position, then the trader can write another call option with the strike
price K and TTM T1 < T ′. While maintaining the long position in the call
option with TTM T ′, the trader can generate income by periodically selling
call options with shorter maturities. In this regard, this strategy resembles the
covered call strategy.
2.19 Strategy: Calendar Put Spread
This is a horizontal spread consisting of a long position in a close to ATM
put option with TTM T ′ and a short position in another put option with the
same strike price K but shorter TTM T < T ′. This is a net debit trade. The
trader’s outlook is neutral to bearish. At the expiration of the short put option
(t = T ), the best case scenario is if the stock price is right at the strike price
13For some literature on calendar/diagonal call/put spreads, see, e.g., Carmona and Durrleman ( 2003),
Carr and Javaheri ( 2005), Dale and Currie ( 2015), Gatheral and Jacquier ( 2014), Kawaller et al. ( 2002),
Liu and T ang ( 2010), Manoliu ( 2004), Pirrong ( 2017), Till ( 2008).

2 Options 15
(ST = K ). At t = T let V be the value of the long put option (expiring at
t = T ′) assuming ST = K .W eh a v e :
Pmax = V − D (2.75)
Lmax = D (2.76)
If at the expiration of the short put option the stock price K ≤ ST ≤
Sstop −loss ,w h e r eSstop −loss is the stop-loss price above which the trader would
unwind the entire position, then the trader can write another put option with
the strike price K and TTM T1 < T ′. While maintaining the long position
in the put option with TTM T ′, the trader can generate income by period-
ically selling put options with shorter maturities. In this regard, this strategy
resembles the covered put strategy.
2.20 Strategy: Diagonal Call Spread
This is a diagonal spread consisting of a long position in a deep ITM call
option with a strike price K1 and TTM T ′, and a short position in an OTM
call option with a strike price K2 and shorter TTM T < T ′. This is a net debit
trade. The trader’s outlook is bullish. At t = T let V be the value of the long
call option (expiring at t = T ′) assuming ST = K .W eh a v e :
Pmax = V − D (2.77)
Lmax = D (2.78)
If at the expiration of the short call option the stock price Sstop −loss ≤ ST ≤
K2,w h e r e Sstop −loss is the stop-loss price below which the trader would
unwind the entire position, then the trader can write another OTM call option
with TTM T1 < T ′. While maintaining the long position in the call option
with TTM T ′, the trader can generate income by periodically selling OTM
call options with shorter maturities. In this regard, this strategy is similar to
the calendar call spread. The main difference is that, in the diagonal call spread
the deep ITM call option (unlike the close to ATM call option in the calendar
call spread) more closely mimics the underlying stock, so the position is more
protected against a sharp rise in the stock price.

16 Z. Kakushadze and J. A. Serur
2.21 Strategy: Diagonal Put Spread
This is a diagonal spread consisting of a long position in a deep ITM put
option with a strike price K1 and TTM T ′, and a short position in an OTM
put option with a strike price K2 and shorter TTM T < T ′. This is a net debit
trade. The trader’s outlook is bearish. At t = T let V be the value of the long
put option (expiring at t = T ′) assuming ST = K .W eh a v e :
Pmax = V − D (2.79)
Lmax = D (2.80)
If at the expiration of the short put option the stock price K2 ≤ ST ≤
Sstop −loss ,w h e r eSstop −loss is the stop-loss price above which the trader would
unwind the entire position, then the trader can write another OTM put option
with TTM T1 < T ′. While maintaining the long position in the put option
with TTM T ′, the trader can generate income by periodically selling OTM
put options with shorter maturities. In this regard, this strategy is similar to
the calendar put spread. The main difference is that, in the diagonal put spread
the deep ITM put option (unlike the close to ATM put option in the calendar
put spread) more closely mimics the underlying stock, so the position is more
protected against a sharp drop in the stock price.
2.22 Strategy: Long Straddle
This is a volatility strategy consisting of a long position in an ATM call option,
and a long position in an ATM put option with a strike price K .T h i si sa
net debit trade. The trader’s outlook is neutral. This is a capital gain strategy.
We have14:
fT = (ST − K )+ + (K − ST )+ − D (2.81)
S∗up = K + D (2.82)
S∗down = K − D (2.83)
14For some literature on straddle/strangle strategies, see, e.g., Copeland and Galai ( 1983), Coval and
Shumway ( 2001), Engle and Rosenberg ( 2000), Gao et al. ( 2017), Goltz and Lai ( 2009), Guo ( 2000),
Hansch et al. ( 1998), Noh et al. ( 1994), Rusnáková and Šoltés ( 2012), Suresh ( 2015). Academic literature
speciﬁcally on long/short guts strategies (which can be thought of as variations on straddles) appears to
be more scarce. For a book reference, see, e.g., Cohen ( 2005). For covered straddles, see, e.g., Johnson
(1979).

2 Options 17
Pmax = unlimited (2.84)
Lmax = D (2.85)
2.23 Strategy: Long Strangle
This is a volatility strategy consisting of a long position in an OTM call option
with a strike price K1, and a long position in an OTM put option with a strike
price K2. This is a net debit trade. However, because both call and put options
are OTM, this strategy is less costly to establish than a long straddle position.
The ﬂipside is that the movement in the stock price required to reach one of
the break-even points is also more signiﬁcant. The trader’s outlook is neutral.
This is a capital gain strategy. We have:
fT = (ST − K1)+ + (K2 − ST )+ − D (2.86)
S∗up = K1 + D (2.87)
S∗down = K2 − D (2.88)
Pmax = unlimited (2.89)
Lmax = D (2.90)
2.24 Strategy: Long Guts
This is a volatility strategy consisting of a long position in an ITM call option
with a strike price K1, and a long position in an ITM put option with a
strike price K2. This is a net debit trade. Since both call and put options are
ITM, this strategy is more costly to establish than a long straddle position. The
trader’s outlook is neutral. This is a capital gain strategy. We have (assuming
D > K2 − K1)15:
fT = (ST − K1)+ + (K2 − ST )+ − D (2.91)
S∗up = K1 + D (2.92)
S∗down = K2 − D (2.93)
Pmax = unlimited (2.94)
Lmax = D − (K2 − K1)( 2.95)
15Otherwise this strategy would generate risk-free proﬁts.

18 Z. Kakushadze and J. A. Serur
2.25 Strategy: Short Straddle
This a is sideways strategy consisting of a short position in an ATM call option,
and a short position in an ATM put option with a strike price K .T h i si sa
net credit trade. The trader’s outlook is neutral. This is an income strategy.
We have:
fT =− (ST − K )+ − (K − ST )+ + C (2.96)
S∗up = K + C (2.97)
S∗down = K − C (2.98)
Pmax = C (2.99)
Lmax = unlimited (2.100)
2.26 Strategy: Short Strangle
This is a sideways strategy consisting of a short position in an OTM call option
with a strike price K1, and a short position in an OTM put option with a strike
price K2. This is a net credit trade. Since both call and put options are OTM,
this strategy is less risky than a short straddle position. The ﬂipside is that the
initial credit is also lower. The trader’s outlook is neutral. This is an income
strategy. We have:
fT =− (ST − K1)+ − (K2 − ST )+ + C (2.101)
S∗up = K1 + C (2.102)
S∗down = K2 − C (2.103)
Pmax = C (2.104)
Lmax = unlimited (2.105)
2.27 Strategy: Short Guts
This is a sideways strategy consisting of a short position in an ITM call option
with a strike price K1, and a short position in an ITM put option with a
strike price K2. This is a net credit trade. Since both call and put options are
ITM, the initial credit is higher than in a short straddle position. The ﬂipside

2 Options 19
is that the risk is also higher. The trader’s outlook is neutral. This is an income
strategy. We have 16:
fT =− (ST − K1)+ − (K2 − ST )+ + C (2.106)
S∗up = K1 + C (2.107)
S∗down = K2 − C (2.108)
Pmax = C − (K2 − K1)( 2.109)
Lmax = unlimited (2.110)
2.28 Strategy: Long Call Synthetic Straddle
This volatility strategy (which is the same as a long straddle with the put
replaced by a synthetic put) amounts to shorting stock and buying two ATM
(or the nearest ITM) call options with a strike price K . The trader’s outlook
is neutral. This is a capital gain strategy. 17 We have (assuming S0 ≥ K and
D > S0 − K ):
fT = S0 − ST + 2 × (ST − K )+ − D (2.111)
S∗up = 2 × K − S0 + D (2.112)
S∗down = S0 − D (2.113)
Pmax = unlimited (2.114)
Lmax = D − (S0 − K )( 2.115)
2.29 Strategy: Long Put Synthetic Straddle
This volatility strategy (which is the same as a long straddle with the call
replaced by a synthetic call) amounts to buying stock and buying two ATM
(or the nearest ITM) put options with a strike price K . The trader’s outlook
is neutral. This is a capital gain strategy. We have (assuming S0 ≤ K and
D > K − S0):
fT = ST − S0 + 2 × (K − ST )+ − D (2.116)
S∗up = S0 + D (2.117)
S∗down = 2 × K − S0 − D (2.118)
16Similarly to long guts, here we assume that C > K2 − K1 .
17Academic literature on synthetic straddles appears to be scarce. See, e.g., T rifonov et al. ( 2011, 2014).

20 Z. Kakushadze and J. A. Serur
Pmax = unlimited (2.119)
Lmax = D − (K − S0)( 2.120)
2.30 Strategy: Short Call Synthetic Straddle
This sideways strategy (which is the same as a short straddle with the put
replaced by a synthetic put) amounts to buying stock and selling two ATM
(or the nearest OTM) call options with a strike price K . The trader’s outlook
is neutral. This is a capital gain strategy. We have (assuming S0 ≤ K ):
fT = ST − S0 − 2 × (ST − K )+ + C (2.121)
S∗up = 2 × K − S0 + C (2.122)
S∗down = S0 − C (2.123)
Pmax = K − S0 + C (2.124)
Lmax = unlimited (2.125)
2.31 Strategy: Short Put Synthetic Straddle
This sideways strategy (which is the same as a short straddle with the call
replaced by a synthetic call) amounts to shorting stock and selling two ATM
(or the nearest OTM) put options with a strike price K . The trader’s outlook
is neutral. This is a capital gain strategy. We have (assuming S0 ≥ K ):
fT = S0 − ST − 2 × (K − ST )+ + C (2.126)
S∗up = S0 + C (2.127)
S∗down = 2 × K − S0 − C (2.128)
Pmax = S0 − K + C (2.129)
Lmax = unlimited (2.130)
2.32 Strategy: Covered Short Straddle
This strategy amounts to augmenting a covered call by writing a put option
with the same strike price K and TTM as the sold call option and thereby

2 Options 21
increasing the income. The trader’s outlook is bullish. We have:
fT = ST − S0 − (ST − K )+ − (K − ST )+ + C (2.131)
S∗ = 1
2 (S0 + K − C) (2.132)
Pmax = K − S0 + C (2.133)
Lmax = S0 + K − C (2.134)
2.33 Strategy: Covered Short Strangle
This strategy amounts to augmenting a covered call by writing an OTM put
option with a strike price K ′ and the same TTM as the sold call option (whose
strike price is K ) and thereby increasing the income. The trader’s outlook is
bullish. We have:
fT = ST − S0 − (ST − K )+ − (K ′ − ST )+ + C (2.135)
Pmax = K − S0 + C (2.136)
Lmax = S0 + K ′ − C (2.137)
2.34 Strategy: Strap
This is a volatility strategy consisting of a long position in two ATM call options,
and a long position in an ATM put option with a strike price K .T h i si san e t
debit trade. The trader’s outlook is bullish. This is a capital gain strategy. We
have
18:
fT = 2 × (ST − K )+ + (K − ST )+ − D (2.138)
S∗up = K + D
2 (2.139)
S∗down = K − D (2.140)
Pmax = unlimited (2.141)
Lmax = D (2.142)
18For some literature on strip and strap strategies, see, e.g., Jha and Kalimipal ( 2010), T opaloglou et al.
(2011).

22 Z. Kakushadze and J. A. Serur
2.35 Strategy: Strip
This is a volatility strategy consisting of a long position in an ATM call option,
and a long position in two ATM put options with a strike price K .T h i si sa
net debit trade. The trader’s outlook is bearish. This is a capital gain strategy.
We have:
fT = (ST − K )+ + 2 × (K − ST )+ − D (2.143)
S∗up = K + D (2.144)
S∗down = K − D
2 (2.145)
Pmax = unlimited (2.146)
Lmax = D (2.147)
2.36 Strategy: Call Ratio Backspread
This strategy consists of a short position in NS close to ATM call options with
as t r i k ep r i c eK1, and a long position in NL OTM call options with a strike
price K2,w h e r e NL > NS . T ypically, NL = 2 and NS = 1,o r NL = 3 and
NS = 2. The trader’s outlook is strongly bullish. This is a capital gain strategy.
We have19:
fT = NL × (ST − K2)+ − NS × (ST − K1)+ − H (2.148)
S∗down = K1 − H/NS , H < 0 (2.149)
S∗up = (NL × K2 − NS × K1 + H )/(NL − NS )( 2.150)
Pmax = unlimited (2.151)
Lmax = NS × (K2 − K1) + H (2.152)
2.37 Strategy: Put Ratio Backspread
This strategy consists of a short position in NS close to ATM put options with
as t r i k ep r i c eK1, and a long position in NL OTM put options with a strike
price K2,w h e r e NL > NS . T ypically, NL = 2 and NS = 1,o r NL = 3 and
19For some literature on call/put ratio (back)spreads, see, e.g., Augustin et al. ( 2015), Chaput and Eder-
ington ( 2008), Šoltés ( 2010), Šoltés and Amaitiek ( 2010b), Šoltés and Rusnáková ( 2013).

2 Options 23
NS = 2. The trader’s outlook is strongly bearish. This is a capital gain strategy.
We have:
fT = NL × (K2 − ST )+ − NS × (K1 − ST )+ − H (2.153)
S∗up = K1 + H/NS , H < 0 (2.154)
S∗down = (NL × K2 − NS × K1 − H )/(NL − NS )( 2.155)
Pmax = NL × K2 − NS × K1 − H (2.156)
Lmax = NS × (K1 − K2) + H (2.157)
2.38 Strategy: Ratio Call Spread
This strategy consists of a short position in NS close to ATM call options with
as t r i k ep r i c eK1, and a long position in NL ITM call options with a strike
price K2,w h e r e NL < NS . T ypically, NL = 1 and NS = 2,o r NL = 2 and
NS = 3.T h i si sa ni n c o m es t r a t e g yi fi ti ss t r u c t u r e da san e tc r e d i tt r a d e .T h e
trader’s outlook is neutral to bearish. We have 20:
fT = NL × (ST − K2)+ − NS × (ST − K1)+ − H (2.158)
S∗down = K2 + H/NL , H > 0 (2.159)
S∗up = (NS × K1 − NL × K2 − H )/(NS − NL )( 2.160)
Pmax = NL × (K1 − K2) − H (2.161)
Lmax = unlimited (2.162)
2.39 Strategy: Ratio Put Spread
This strategy consists of a short position in NS close to ATM put options with
as t r i k ep r i c eK1, and a long position in NL ITM put options with a strike
price K2,w h e r e NL < NS . T ypically, NL = 1 and NS = 2,o r NL = 2 and
NS = 3.T h i si sa ni n c o m es t r a t e g yi fi ti ss t r u c t u r e da san e tc r e d i tt r a d e .T h e
trader’s outlook is neutral to bullish. We have:
fT = NL × (K2 − ST )+ − NS × (K1 − ST )+ − H (2.163)
S∗up = K2 − H/NL , H > 0 (2.164)
S∗down = (NS × K1 − NL × K2 + H )/(NS − NL )( 2.165)
20So, the difference between call/put ratio backspreads and ratio call/put spreads is that in the former
NL > NS , while in the latter NL < NS .

24 Z. Kakushadze and J. A. Serur
Pmax = NL × (K2 − K1) − H (2.166)
Lmax = NS × K1 − NL × K2 + H (2.167)
2.40 Strategy: Long Call Butterfly
This is a sideways strategy consisting of a long position in an OTM call option
with a strike price K1, a short position in two ATM call options with a strike
price K2, and a long position in an ITM call option with a strike price K3.
The strikes are equidistant: K2 − K3 = K1 − K2 = κ. This is a relatively
low cost net debit trade. The trader’s outlook is neutral. This is a capital gain
strategy. We have
21:
fT = (ST − K1)+ + (ST − K3)+ − 2 × (ST − K2)+ − D (2.168)
S∗down = K3 + D (2.169)
S∗up = K1 − D (2.170)
Pmax = κ − D (2.171)
Lmax = D (2.172)
Strategy: Modified Call Butterfly
This is a variation of the long call butterﬂy strategy where the strikes are no
longer equidistant; instead we have K1 − K2 < K2 − K3. This results in a
sideways strategy with a bullish bias. We have:
fT = (ST − K1)+ + (ST − K3)+ − 2 × (ST − K2)+ − D (2.173)
S∗ = K3 + D (2.174)
Pmax = K2 − K3 − D (2.175)
Lmax = D (2.176)
21For some literature on butterﬂy spreads (including iron butterﬂies), see, e.g., Balbás et al. ( 1999),
Howison et al. ( 2013), Jongadsayakul ( 2017), Matsypura and Timkovsky ( 2010), Youbi et al. ( 2017),
Wolf ( 2014), Wystup ( 2017). Academic literature on condor strategies (which can be thought of as
variations on butterﬂies) appears to be more scarce. See, e.g., Niblock ( 2017).

2 Options 25
2.41 Strategy: Long Put Butterfly
This is a sideways strategy consisting of a long position in an OTM put option
with a strike price K1, a short position in two ATM put options with a strike
price K2, and a long position in an ITM put option with a strike price K3.
The strikes are equidistant: K3 − K2 = K2 − K1 = κ. This is a relatively
low cost net debit trade. The trader’s outlook is neutral. This is a capital gain
strategy. We have:
fT = (K1 − ST )+ + (K3 − ST )+ − 2 × (K2 − ST )+ − D (2.177)
S∗up = K3 − D (2.178)
S∗down = K1 + D (2.179)
Pmax = κ − D (2.180)
Lmax = D (2.181)
Strategy: Modified Put Butterfly
This is a variation of the long put butterﬂy strategy where the strikes are no
longer equidistant; instead we have K3 − K2 < K2 − K1.T h i sr e s u l t si n
a sideways strategy with a bullish bias. We have (for H > 0 there is also
S∗up = K3 − H )22:
fT = (K1 − ST )+ + (K3 − ST )+ − 2 × (K2 − ST )+ − H (2.182)
S∗down = 2 × K2 − K3 + H (2.183)
Pmax = K3 − K2 − H (2.184)
Lmax = 2 × K2 − K1 − K3 + H (2.185)
2.42 Strategy: Short Call Butterfly
This is a volatility strategy consisting of a short position in an ITM call option
with a strike price K1, a long position in two ATM call options with a strike
price K2, and a short position in an OTM call option with a strike price K3.
The strikes are equidistant: K3 − K2 = K2 − K1 = κ. This is a net credit
trade. In this sense, this is an income strategy. However, the potential reward
22Ideally, this should be structured as a net credit trade, albeit this may not always be possible.

26 Z. Kakushadze and J. A. Serur
is sizably smaller than with a short straddle or a short strangle (albeit with a
lower risk). The trader’s outlook is neutral. We have:
fT = 2 × (ST − K2)+ − (ST − K1)+ − (ST − K3)+ + C (2.186)
S∗up = K3 − C (2.187)
S∗down = K1 + C (2.188)
Pmax = C (2.189)
Lmax = κ − C (2.190)
2.43 Strategy: Short Put Butterfly
This is a volatility strategy consisting of a short position in an ITM put option
with a strike price K1, a long position in two ATM put options with a strike
price K2, and a short position in an OTM put option with a strike price K3.
The strikes are equidistant: K2 − K3 = K1 − K2 = κ. This is a net credit
trade. In this sense, this is an income strategy. However, the potential reward
is sizably smaller than with a short straddle or a short strangle (albeit with a
lower risk). The trader’s outlook is neutral. We have:
fT = 2 × (K2 − ST )+ − (K1 − ST )+ − (K3 − ST )+ + C (2.191)
S∗down = K3 + C (2.192)
S∗up = K1 − C (2.193)
Pmax = C (2.194)
Lmax = κ − C (2.195)
2.44 Strategy: ‘‘Long’’ Iron Butterfly
This sideways strategy is a combination of a bull put spread and a bear call
spread and consists of a long position in an OTM put option with a strike price
K1, a short position in an ATM put option and an ATM call option with a
strike price K2, and a long position in an OTM call option with a strike price
K3. The strikes are equidistant: K2 − K1 = K3 − K2 = κ. This is a net credit
trade. The trader’s outlook is neutral. This is an income strategy. We have:
fT = (K1 − ST )+ − (K2 − ST )+ − (ST − K2 )+ + (ST − K3)+ + C (2.196)
S∗up = K2 + C (2.197)
S∗down = K2 − C (2.198)

2 Options 27
Pmax = C (2.199)
Lmax = κ − C (2.200)
2.45 Strategy: ‘‘Short’’ Iron Butterfly
This volatility strategy is a combination of a bear put spread and a bull call
spread and consists of a short position in an OTM put option with a strike
price K1, a long position in an ATM put option and an ATM call option with
as t r i k ep r i c eK2, and a short position in an OTM call option with a strike
price K3. The strikes are equidistant: K2 − K1 = K3 − K2 = κ.T h i si sa
net debit trade. The trader’s outlook is neutral. This is a capital gain strategy.
We have:
fT = (K2 − ST )+ + (ST − K2 )+ − (K1 − ST )+ − (ST − K3)+ − D (2.201)
S∗up = K2 + D (2.202)
S∗down = K2 − D (2.203)
Pmax = κ − D (2.204)
Lmax = D (2.205)
2.46 Strategy: Long Call Condor
This is a sideways strategy consisting of a long position in an ITM call option
with a strike price K1, a short position in an ITM call option with a higher
strike price K2, a short position in an OTM call option with a strike price
K3, and a long position in an OTM call option with a higher strike price K4.
All strikes are equidistant: K4 − K3 = K3 − K2 = K2 − K1 = κ.T h i si s
a relatively low cost net debit trade. The trader’s outlook is neutral. This is a
capital gain strategy. We have:
fT = (ST − K1)+ − (ST − K2 )+ − (ST − K3)+ + (ST − K4 )+ − D (2.206)
S∗up = K4 − D (2.207)
S∗down = K1 + D (2.208)
Pmax = κ − D (2.209)
Lmax = D (2.210)

28 Z. Kakushadze and J. A. Serur
2.47 Strategy: Long Put Condor
This is a sideways strategy consisting of a long position in an OTM put option
with a strike price K1, a short position in an OTM put option with a higher
strike price K2, a short position in an ITM put option with a strike price K3,
and a long position in an ITM put option with a higher strike price K4.A l l
strikes are equidistant: K4 − K3 = K3 − K2 = K2 − K1 = κ.T h i si sa
relatively low cost net debit trade. The trader’s outlook is neutral. This is a
capital gain strategy. We have:
fT = (K1 − ST )+ − (K2 − ST )+ − (K3 − ST )+ + (K4 − ST )+ − D (2.211)
S∗up = K4 − D (2.212)
S∗down = K1 + D (2.213)
Pmax = κ − D (2.214)
Lmax = D (2.215)
2.48 Strategy: Short Call Condor
This is a volatility strategy consisting of a short position in an ITM call option
with a strike price K1, a long position in an ITM call option with a higher
strike price K2, a long position in an OTM call option with a strike price K3,
and a short position in an OTM call option with a higher strike price K4.
All strikes are equidistant: K4 − K3 = K3 − K2 = K2 − K1 = κ.T h i si s
a relatively low net credit trade. As with a short call butterﬂy, the potential
reward is sizably smaller than with a short straddle or a short strangle (albeit
with a lower risk). So, this is a capital gain (rather than an income) strategy.
The trader’s outlook is neutral. We have:
fT = (ST − K2 )+ + (ST − K3)+ − (ST − K1)+ − (ST − K4 )+ + C (2.216)
S∗up = K4 − C (2.217)
S∗down = K1 + C (2.218)
Pmax = C (2.219)
Lmax = κ − C (2.220)
2.49 Strategy: Short Put Condor
This is a volatility strategy consisting of a short position in an OTM put option
with a strike price K1, a long position in an OTM put option with a higher
strike price K2, a long position in an ITM put option with a strike price K3,
and a short position in an ITM put option with a higher strike price K4.A l l

2 Options 29
strikes are equidistant: K4 − K3 = K3 − K2 = K2 − K1 = κ.T h i si s
a relatively low net credit trade. As with a short put butterﬂy, the potential
reward is sizably smaller than with a short straddle or a short strangle (albeit
with a lower risk). So, this is a capital gain (rather than an income) strategy.
The trader’s outlook is neutral. We have:
fT = (K2 − ST )+ + (K3 − ST )+ − (K1 − ST )+ − (K4 − ST )+ + C (2.221)
S∗up = K4 − C (2.222)
S∗down = K1 + C (2.223)
Pmax = C (2.224)
Lmax = κ − C (2.225)
2.50 Strategy: Long Iron Condor
This sideways strategy is a combination of a bull put spread and a bear call
spread and consists of a long position in an OTM put option with a strike
price K
1, a short position in an OTM put option with a higher strike price
K2, a short position in an OTM call option with a strike price K3,a n dal o n g
position in an OTM call option with a higher strike price K4. The strikes are
equidistant: K4 − K3 = K3 − K2 = K2 − K1 = κ. This is a net credit trade.
The trader’s outlook is neutral. This is an income strategy. We have:
fT = (K1 − ST )+ + (ST − K4 )+ − (K2 − ST )+ − (ST − K3)+ + C (2.226)
S∗up = K3 + C (2.227)
S∗down = K2 − C (2.228)
Pmax = C (2.229)
Lmax = κ − C (2.230)
2.51 Strategy: Short Iron Condor
This volatility strategy is a combination of a bear put spread and a bull call
spread and consists of a short position in an OTM put option with a strike
price K1, a long position in an OTM put option with a higher strike price
K2, a long position in an OTM call option with a strike price K3,a n das h o r t
position in an OTM call option with a higher strike price K4. The strikes are
equidistant: K4 − K3 = K3 − K2 = K2 − K1 = κ. This is a net debit trade.
The trader’s outlook is neutral. This is a capital gain strategy. We have:
fT = (K2 − ST )+ + (ST − K3)+ − (K1 − ST )+ − (ST − K4 )+ − D (2.231)
S∗up = K3 + D (2.232)

30 Z. Kakushadze and J. A. Serur
S∗down = K2 − D (2.233)
Pmax = κ − D (2.234)
Lmax = D (2.235)
2.52 Strategy: Long Box
This volatility strategy can be viewed as a combination of a long synthetic
forward and a short synthetic forward, or as a combination of a bull call spread
and a bear put spread, and consists of a long position in an ITM put option
with a strike price K
1, a short position in an OTM put option with a lower
strike price K2, a long position in an ITM call option with the strike price
K2, and a short position in an OTM call option with the strike price K1.T h e
trader’s outlook is neutral. This is a capital gain strategy. 23 We have (assuming
K1 ≥ K2 + D):
fT = (K1 − ST )+ − (K2 − ST )+ + (ST − K2)+ − (ST − K1)+ − D
= K1 − K2 − D (2.236)
Pmax = (K1 − K2) − D (2.237)
2.53 Strategy: Collar
This strategy (a.k.a. “fence”) is a covered call augmented by a long put option as
insurance against the stock price falling. 24 It amounts to buying stock, buying
an OTM put option with a strike price K1, and selling an OTM call option
with a higher strike price K2. The trader’s outlook is moderately bullish. This
is a capital gain strategy. We have 25:
fT = ST − S0 + (K1 − ST )+ − (ST − K2)+ − H (2.238)
S∗ = S0 + H (2.239)
23In some cases it can be used as a tax strategy—see, e.g., Cohen ( 2005). For some literature on box
option strategies, see, e.g., BenZion et al. ( 2005), Bharadwaj and Wiggins ( 2001), Billingsley and Chance
(1985), Clarke et al. ( 2013), Fung et al. ( 2004), Hemler and Miller ( 1997), Jongadsayakul ( 2016), Ronn
a n dR o n n(1989), Vipul ( 2009).
24Similarly, a short collar is a covered put augmented by a long call option.
25For some literature on collar strategies, see, e.g., Bartonová ( 2012), Burnside et al. ( 2011), D’Antonio
(2008), Israelov and Klein ( 2016), Li and Yang ( 2017), Ofﬁcer ( 2004, 2006), Shan et al. ( 2010), Szado
and Schneeweis ( 2010, 2011), Timmermans et al. ( 2017), Yim et al. ( 2011).

2 Options 31
Pmax = K2 − S0 − H (2.240)
Lmax = S0 − K1 + H (2.241)
2.54 Strategy: Bullish Short Seagull Spread
This option trading strategy is a bull call spread ﬁnanced with a sale of an
OTM put option. It amounts to a short position in an OTM put option with
as t r i k ep r i c eK1, a long position in an ATM call option with a strike price K2,
and a short position in an OTM call option with a strike price K3. Ideally, the
trade should be structured to have zero cost. The trader’s outlook is bullish.
This is a capital gain strategy. We have 26:
fT =− (K1 − ST )+ + (ST − K2)+ − (ST − K3)+ − H (2.242)
S∗ = K2 + H, H > 0 (2.243)
S∗ = K1 + H, H < 0 (2.244)
K1 ≤ S∗ ≤ K2, H = 0 (2.245)
Pmax = K3 − K2 − H (2.246)
Lmax = K1 + H (2.247)
2.55 Strategy: Bearish Long Seagull Spread
This option trading strategy is a short combo (short risk reversal) hedged against
the stock price rising by buying an OTM call option. It amounts to a long
position in an OTM put option with a strike price K
1, a short position in
an ATM call option with a strike price K2, and a long position in an OTM
call option with a strike price K3. Ideally, the trade should be structured to
have zero cost. The trader’s outlook is bearish. This is a capital gain strategy.
We have:
fT = (K1 − ST )+ − (ST − K2)+ + (ST − K3)+ − H (2.248)
S∗ = K1 − H, H > 0 (2.249)
S∗ = K2 − H, H < 0 (2.250)
K1 ≤ S∗ ≤ K2, H = 0 (2.251)
26Academic literature on seagull spreads appears to be scarce. For a book reference, see, e.g., Wystup
(2017).

32 Z. Kakushadze and J. A. Serur
Pmax = K1 − H (2.252)
Lmax = K3 − K2 + H (2.253)
2.56 Strategy: Bearish Short Seagull Spread
This option trading strategy is a bear put spread ﬁnanced with a sale of an
OTM call option. It amounts to a short position in an OTM put option with
as t r i k ep r i c eK1, a long position in an ATM put option with a strike price K2,
and a short position in an OTM call option with a strike price K3. Ideally, the
trade should be structured to have zero cost. The trader’s outlook is bearish.
This is a capital gain strategy. We have:
fT =− (K1 − ST )+ + (K2 − ST )+ − (ST − K3)+ − H (2.254)
S∗ = K2 − H, H > 0 (2.255)
S∗ = K3 − H, H < 0 (2.256)
K2 ≤ S∗ ≤ K3, H = 0 (2.257)
Pmax = K2 − K1 − H (2.258)
Lmax = unlimited (2.259)
2.57 Strategy: Bullish Long Seagull Spread
This option trading strategy is a long combo (long risk reversal) hedged against
the stock price falling by buying an OTM put option. It amounts to a long
position in an OTM put option with a strike price K1, a short position in
an ATM put option with a strike price K2, and a long position in an OTM
call option with a strike price K3. Ideally, the trade should be structured to
have zero cost. The trader’s outlook is bullish. This is a capital gain strategy.
We have:
fT = (K1 − ST )+ − (K2 − ST )+ + (ST − K3)+ − H (2.260)
S∗ = K3 + H, H > 0 (2.261)
S∗ = K2 + H, H < 0 (2.262)
K2 ≤ S∗ ≤ K3, H = 0 (2.263)
Pmax = unlimited (2.264)
Lmax = K2 − K1 + H (2.265)

2 Options 33
References
Amaitiek, O. F . S., Bálint, T ., & Rešovský, M. (2010). The Short Call Ladder Strategy
and Its Application in T rading and Hedging. Acta Montanistica Slovaca , 15(3),
171–182.
Andersen, L. (1999). A Simple Approach to the Pricing of Bermudan Swaptions in the
Multi-factor Libor Market Model. Journal of Computational Finance , 3(2), 5–32.
Augustin, P ., Brenner, B., & Subrahmanyam, M. G. (2015). Informed Options T rading
prior to M&A Announcements: Insider T rading? (Working Paper). Available online:
https://ssrn.com/abstract=2441606.
Balbás, A., Longarela, I. R., & Lucia, J. J. (1999). How Financial Theory Applies
to Catastrophe-Linked Derivatives—An Empirical T est of Several Pricing Models.
Journal of Risk and Insurance , 66 (4), 551–582.
Bartonová, M. (2012). Hedging of Sales by Zero-Cost Collar and Its Financial Impact.
Journal of Competitiveness , 4 (2), 111–127.
Baxter, M., & Rennie, A. (1996). Financial Calculus: An Introduction to Derivative
Pricing. Cambridge, UK: Cambridge University Press.
Benavides, G. (2009). Predictive Accuracy of Futures Options Implied Volatility: The
Case of the Exchange Rate Futures Mexican Peso-US Dollar. Panorama Económico,
5(9), 55–95.
BenZion, U., Anan, S. D., & Yagil, J. (2005). Box Spread Strategies and Arbitrage
Opportunities. Journal of Derivatives , 12(3), 47–62.
Bharadwaj, A., & Wiggins, J. B. (2001). Box Spread and Put-Call Parity T ests for the
S&P 500 Index LEAPS Market. Journal of Derivatives , 8(4), 62–71.
Billingsley, R. S., & Chance, D. M. (1985). Options Market Efﬁciency and the Box
Spread Strategy. Financial Review , 20(4), 287–301.
Black, F ., & Scholes, M. (1973). The Pricing of Options and Corporate Liabilities.
Journal of Political Economy , 81(3), 637–659.
Bozic, M., & Fortenbery, T . R. (2012). Creating Synthetic Cheese Futures: A Method
for Matching Cash and Futures Prices in Dairy. Journal of Agribusiness , 30(2),
87–102.
Breeden, D. T ., & Litzenberger, R. H. (1978). Prices of State-Contingent Claims
Implicit in Option Prices. Journal of Business , 51(4), 621–651.
Burnside, C., Eichenbaum, M., Kleshchelski, I., & Rebelo, S. (2011). Do Peso Prob-
lems Explain the Returns to the Carry T rade? Review of Financial Studies , 24 (3),
853–891.
Carmona, R., & Durrleman, V . (2003). Pricing and Hedging Spread Options. SIAM
Review, 45(4), 627–685.
Carr, P ., & Javaheri, A. (2005). The Forward PDE for European Options on Stocks
with Fixed Fractional Jumps. International Journal of Theoretical and Applied
Finance, 8(2), 239–253.
Cartea, A., & Pedraz, C. G. (2012). How Much Should We Pay for Interconnecting
Electricity Markets? A Real Options Approach. Energy Economics , 34 (1), 14–30.

34 Z. Kakushadze and J. A. Serur
Chaput, J. S., & Ederington, L. H. (2003). Option Spread and Combination T rading.
Journal of Derivatives , 10(4), 70–88.
Chaput, J. S., & Ederington, L. H. (2005). Vertical Spread Design. Journal of Deriva-
tives, 12(3), 28–46.
Chaput, J. S., & Ederington, L. H. (2008). Ratio Spreads. Journal of Derivatives ,
15(3), 41–57.
Che, Y. S. (2016). A Study on the Risk and Return of Option Writing Strategies . Ph.D.
thesis, HKBU Institutional Repository. Open Access Theses and Dissertations.
187. Hong Kong Baptist University, Hong Kong, China. Available online: https://
repository.hkbu.edu.hk/etd_oa/187/.
Che, S. Y. S., & Fung, J. K. W . (2011). The Performance of Alternative Futures
Buy-Write Strategies. Journal of Futures Markets , 31(12), 1202–1227.
Chen, A. H. Y., Chen, K. C., & Howell, S. (1999). An Analysis of Dividend Enhanced
Convertible Stocks. International Review of Economics and Finance , 8(3), 327–338.
Clarke, R. G., de Silva, H., & Thorley, S. (2013). Fundamentals of Futures and Options .
New York, NY: The Research Foundation of CFA Institute.
Cohen, G. (2005). The Bible of Options Strategies: The Deﬁnitive Guide for Practical
T rading Strategies. Upper Saddle River, NJ: Financial Times Prentice Hall.
Cong, J., T an, K. S., & Weng, C. (2013). VAR-Based Optimal Partial Hedging. ASTIN
Bulletin: The Journal of the IAA , 43(3), 271–299.
Cong, J., T an, K. S., & Weng, C. (2014). CVaR-Based Optimal Partial Hedging.
Journal of Risk , 16 (3), 49–83.
Copeland, T . E., & Galai, D. (1983). Information Effects on the Bid-Ask Spread.
Journal of Finance , 38(5), 1457–1469.
Coval, J. D., & Shumway, T . (2001). Expected Options Returns. Journal of Finance ,
56 (3), 983–1009.
Dale, A., & Currie, E. (2015). An Alternative Funding Model for Agribusiness
Research in Canada. Agricultural Sciences, 6 (9), 961–969.
D’Antonio, L. (2008). Equity Collars as Alternative to Asset Allocation. Journal of
Financial Service Professionals , 62(1), 67–76.
DeMaskey, A. L. (1995). A Comparison of the Effectiveness of Currency Futures
and Currency Options in the Context of Foreign Exchange Risk Management.
Managerial Finance , 21(4), 40–51.
Ebrahim, S., & Rahman, S. (2005). On the Pareto-optimality of Futures Contracts
over Islamic Forward Contracts: Implications for the Emerging Muslim Economies.
Journal of Economic Behavior & Organization , 56 (2), 273–295.
Engle, R., & Rosenberg, J. (2000). T esting the Volatility T erm Structure Using Option
Hedging Criteria. Journal of Derivatives , 8(1), 10–28.
Fabozzi, F . J. (Ed.). (2002). The Handbook of Financial Instruments . Hoboken, NJ:
Wiley.
Feldman, B., & Roy, D. (2004). Passive Options-Based Investment Strategies: The
Case of the CBOE S&P 500 BuyWrite Index. ETF and Indexing ,
38(1), 72–89.
Figlewski, S., Chidambaran, N. K., & Kaplan, S. (1993). Evaluating the Performance
of the Protective Put Strategy. Financial Analysts Journal , 49 (4), 46–56, 69.

2 Options 35
Fung, J. K. W ., Mok, H. M. K., & Wong, K. C. K. (2004). Pricing Efﬁciency in a
Thin Market with Competitive Market Makers: Box Spread Strategies in the Hang
Seng Index Options Market. Financial Review , 39 (3), 435–454.
Gao, C., Xing, Y., & Zhang, X. (2017). Anticipating Uncertainty: Straddles Around
Earnings Announcements (Working Paper). Available online: https://ssrn.com/
abstract=2204549.
Gatheral, J., & Jacquier, A. (2014). Arbitrage-Free SVI Volatility Surfaces. Quantitative
Finance, 14 (1), 59–71.
Goltz, F ., & Lai, W . N. (2009). Empirical Properties of Straddle Returns. Journal of
Derivatives, 17 (1), 38–48.
Guo, D. (2000). Dynamic Volatility T rading Strategies in the Currency Option Mar-
ket. Review of Derivatives Research , 4 (2), 133–154.
Hansch, O., Naik, N. Y., & Viswanathan, S. (1998). Do Inventories Matter in Deal-
ership Markets? Evidence from the London Stock Exchange. Journal of Finance ,
53(5), 1623–1656.
Harˇcariková, M., & Šoltés, M. (2016). Risk Management in Energy Sector Using
Short Call Ladder Strategy. Montenegrin Journal of Economics , 12(3), 39–54.
Harrison, J. M., & Pliska, S. R. (1981). Martingales and Stochastic Integrals in the
Theory of Continuous T rading. Stochastic Processes and Their Applications , 11(3),
215–260.
Haug, E. G. (2001). Closed form Valuation of American Barrier Options. International
Journal of Theoretical and Applied Finance , 4 (2), 355–359.
He, J., T ang, Q., & Zhang, H. (2016). Risk Reducers in Convex Order. Insurance:
Mathematics and Economics , 70, 80–88.
Hemler, M. L., & Miller, T . W ., Jr. (1997). Box Spread Arbitrage Proﬁts Following the
1987 Market Crash. Journal of Financial and Quantitative Analysis , 32(1), 71–90.
Hemler, M. L., & Miller, T . W ., Jr. (2015). The Performance of Options-Based Invest-
ment Strategies: Evidence for Individual Stocks During 2003–2013 (Working Paper).
Available online: http://www.optionseducation.org/content/dam/oic/documents/
literature/ﬁles/perf-options-strategies.pdf .
Henrard, M. P . A. (2006). A Semi-explicit Approach to Canary Swaptions in HJM
One-Factor Model. Applied Mathematical Finance , 13(1), 1–18.
Hill, J. M., Balasubramanian, V ., Gregory, K., & Tierens, I. (2006). Finding Alpha
via Covered Call Writing. Financial Analysts Journal , 62(5), 29–46.
Howison, S. D., Reisinger, C., & Witte, J. H. (2013). The Effect of Nonsmooth
Payoffs on the Penalty Approximation of American Options. SIAM Journal on
Financial Mathematics , 4 (1), 539–574.
Hull, J. C. (2012). Options, Futures and Other Derivatives . Upper Saddle River, NJ:
Prentice Hall.
Israelov, R. (2017). Pathetic Protection: The Elusive Beneﬁts of Protective Puts (Working
Paper). Available online: https://ssrn.com/abstract=2934538.
Israelov, R., & Klein, M. (2016). Risk and Return of Equity Index Collar Strategies.
Journal of Alternative Investments , 19 (1), 41–54.

36 Z. Kakushadze and J. A. Serur
Israelov, R., & Nielsen, L. N. (2014). Covered Call Strategies: One Fact and Eight
Myths. Financial Analysts Journal , 70(6), 23–31.
Israelov, R., & Nielsen, L. N. (2015a). Covered Calls Uncovered. Financial Analysts
Journal, 71(6), 44–57.
Israelov, R., & Nielsen, L. N. (2015b). Still Not Cheap: Portfolio Protection in Calm
Markets. Journal of Portfolio Management , 41(4), 108–120.
Israelov, R., Nielsen, L. N., & Villalon, D. (2017). Embracing Downside Risk. Journal
of Alternative Investments , 19 (3), 59–67.
Jabbour, G., & Budwick, P . (2010). The Option T rader Handbook: Strategies and T rade
Adjustments (2nd ed.). Hoboken, NJ: Wiley.
Jha, R., & Kalimipal, M. (2010). The Economic Signiﬁcance of Conditional Skewness
in Index Option Markets. Journal of Futures Markets , 30(4), 378–406.
Johnson, H. F . (1979). Is It Better to Go Naked on the Street? A Primer on the Options
Market. Notre Dame Lawyer (Notre Dame Law Review) , 55(1), 7–32.
Jongadsayakul, W . (2016). A Box Spread T est of the SET50 Index Options Market
Efﬁciency: Evidence from the Thailand Futures Exchange. International Journal of
Economics and Financial Issues , 6 (4), 1744–1749.
Jongadsayakul, W . (2017). Arbitrage Opportunity in Thailand Futures Exchange: An
Empirical Study of SET50 Index Options (2017 IACB, ICE & ISEC Proceedings,
Paper No. 381). Littleton, CO: Clute Institute.
Kakushadze, Z. (2015). Phynance. Universal Journal of Physics and Application , 9 (2):
64–133. Available online: https://ssrn.com/abstract=2433826.
Kapadia, N., & Szado, E. (2007). The Risk Return Characteristics of the Buy-Write
Strategy on the Russell 2000 Index. Journal of Alternative Investments , 9 (4), 39–56.
Kawaller, I. G., Koch, P . D., & Ludan, L. (2002). Calendar Spreads, Outright Futures
Positions and Risk. Journal of Alternative Investments , 5(3), 59–74.
Kim, I. J. (1990). The Analytic Valuation of American Options. Review of Financial
Studies, 3(4), 547–572.
Li, P ., & Yang, J. (2017). Pricing Collar Options with Stochastic Volatility. Discrete
Dynamics in Nature and Society , 2017, 9673630.
Liu, P ., & T ang, K. (2010). No-Arbitrage Conditions for Storable Commodities and
the Models of Futures T erm Structures.Journal of Banking & Finance , 34 (7), 1675–
1687.
Manoliu, M. (2004). Storage Options Valuation Using Multilevel T rees and Calendar
Spreads. International Journal of Theoretical and Applied Finance , 7 (4), 425–464.
Matsypura, D., & Timkovsky, V . G. (2010). Combinatorics of Option Spreads: The
Margining Aspect (Working Paper). Available online: https://ses.library.usyd.edu.
au/bitstream/2123/8172/1/OMWP_2010_04.pdf .
McMillan, L. G. (2002). Options as a Strategic Investment (4th ed.). New York, NY:
New York Institute of Finance.
Mugwagwa, T ., Ramiah, V ., Naughton, T ., & Moosa, I. (2012). The Efﬁciency of
the Buy-Write Strategy: Evidence from Australia. Journal of International Financial
Markets, Institutions and Money , 22(2), 305–328.

2 Options 37
Nandy (Pal), S., & Chattopadhyay, A. Kr. (2016). Impact of Individual Stock Deriva-
tives Introduction in India on Its Underlying Spot Market Volatility. Asia-Paciﬁc
Journal of Management Research and Innovation , 12(2), 109–133.
Niblock, S. J. (2017). Flight of the Condors: Evidence on the Performance of Condor
Option Spreads in Australia. Applied Finance Letters , 6 (1), 38–53.
Noh, J., Engle, R. F ., & Kane, A. (1994). Forecasting Volatility and Option Prices of
the S&P500 Index. Journal of Derivatives , 2(1), 17–30.
Ofﬁcer, M. S. (2004). Collars and Renegotiation in Mergers and Acquisitions. Journal
of Finance , 59 (6), 2719–2743.
Ofﬁcer, M. S. (2006). The Market Pricing of Implicit Options in Merger Collars.
Journal of Business , 79 (1), 115–136.
Pirrong, C. (2017). The Economics of Commodity Market Manipulation: A Survey.
Journal of Commodity Markets , 5, 1–17.
Pounds, H. (1978). Covered Call Option Writing Strategies and Results. Journal of
Portfolio Management, 4 (2), 31–42.
Rogers, L. C. G., & Shi, Z. (1995). The Value of an Asian Option. Journal of Applied
Probability, 32(4), 1077–1088.
Ronn, A. G., & Ronn, E. I. (1989). The Box Spread Arbitrage Conditions: Theory,
T ests, and Investment Strategies. Review of Financial Studies , 2(1), 91–108.
Rusnáková, M., & Šoltés, V . (2012). Long Strangle Strategy Using Barrier Options
and Its Application in Hedging. Actual Problems of Economics , 134 (8), 452–465.
Rusnáková, M., Šoltés, V ., & Szabo, Z. K. (2015). Short Combo Strategy Using
Barrier Options and Its Application in Hedging. Procedia Economics and Finance ,
32, 166–179.
Shah, A. (2017). Hedging of a Portfolio of Rainfall Insurances Using Rainfall Bonds and
European Call Options (Bull Spread) (Working Paper). Available online: https://
ssrn.com/abstract=2778647.
Shan, L., Garvin, M. J., & Kumar, R. (2010). Collar Options to Manage Revenue
Risks in Real T oll Public-Private Partnership T ransportation Projects. Construction
Management and Economics , 28(10), 1057–1069.
Šoltés, M. (2010). Relationship of Speed Certiﬁcates and Inverse Vertical Ratio Call
Back Spread Option Strategy. E+M Ekonomie a Management , 13(2), 119–124.
Šoltés, V . (2011). The Application of the Long and Short Combo Option Strategies in
the Building of Structured Products. In A. Kocourek (Ed.), Proceedings of the 10th
International Conference: Liberec Economic Forum 2011 (pp. 481–487). Liberec,
Czech Republic: T echnical University of Liberec.
Šoltés, V ., & Amaitiek, O. F . S. (2010a). The Short Put Ladder Strategy and Its Appli-
cation in T rading and Hedging. Club of Economics in Miskolc: Theory, Methodology,
Practice, 6 (2), 77–85.
Šoltés, V ., & Amaitiek, O. F . S. (2010b). Inverse Vertical Ratio Put Spread Strategy
and Its Application in Hedging Against a Price Drop. Journal of Advanced Studies
in Finance , 1(1), 100–107.

38 Z. Kakushadze and J. A. Serur
Šoltés, V ., & Rusnáková, M. (2012). Long Combo Strategy Using Barrier Options
and Its Application in Hedging Against a Price Drop. Acta Montanistica Slovaca ,
17 (1), 17–32.
Šoltés, V ., & Rusnáková, M. (2013). Hedging Against a Price Drop Using the Inverse
Vertical Ratio Put Spread Strategy Formed by Barrier Options. Engineering Eco-
nomics, 24 (1), 18–27.
Stoll, H. R. (1969). The Relationship Between Put and Call Option Prices. Journal of
Finance, 24 (5), 801–824.
Suresh, A. S. (2015). Analysis of Option Combination Strategies. Management Insight,
11(1), 31–40.
Szado, E., & Schneeweis, T . (2010). Loosening Your Collar: Alternative Implemen-
tations of QQQ Collars. Journal of T rading, 5(2), 35–56.
Szado, E., & Schneeweis, T . (2011). An Update of ‘Loosening Your Collar: Alterna-
tive Implementations of QQQ Collars ’: Credit Crisis and Out-of-Sample Performance
(Working Paper). Available online: http://ssrn.com/abstract=1507991.
The Options Institute. (1995). Options: Essential Concepts and T rading Strategies (2nd
ed.). Chicago, IL: Richard D. Irwin Inc.
Till, H. (2008). Case Studies and Risk Management Lessons in Commodity Deriva-
tives T rading. In H. Geman (Ed.), Risk Management in Commodity Markets: From
Shipping to Agriculturals and Energy (pp. 255–291). Chichester, UK: Wiley.
Timmermans, S. H. J. T ., Schumacher, J. M., & Ponds, E. H. M. (2017). A Multi-
objective Decision Framework for Lifecycle Investment (Working Paper). Available
online: http://ssrn.com/abstract=3038803.
T okic, D. (2013). Crude Oil Futures Markets: Another Look into T raders’ Positions.
Journal of Derivatives & Hedge Funds , 19 (4), 321–342.
T opaloglou, N., Vladimirou, H., & Zenios, S. A. (2011). Optimizing International
Portfolios with Options and Forwards. Journal of Banking & Finance , 35(12),
3188–3201.
T rifonov, Y., Yashin, S., Koshelev, E., & Podshibyakin, D. (2011). Application of
Synthetic Straddles for Equity Risk Management. In Z. ˇCernák (Ed.), Materiály VII
mezinárodní vˇedecko – praktická konference “Zprávy vˇ edecké ideje – 2011” .P r a g u e ,
Czech Republic: Education and Science.
T rifonov, Y., Yashin, S., Koshelev, E., & Podshibyakin, D. (2014). T esting the T ech-
nology of Synthetic Straddles (Working Paper). Available online: https://ssrn.com/
abstract=2429657.
Vipul. (2009). Box-Spread Arbitrage Efﬁciency of Nifty Index Options: The Indian
Evidence. Journal of Futures Markets , 29 (6), 544–562.
Whaley, R. E. (2002). Return and Risk of CBOE Buy Write Monthly Index. Journal
of Derivatives , 10(2), 35–42.
Wolf, V . (2014). Comparison of Markovian Price Processes and Optimality of Pay-
offs. Ph.D. thesis, Albert-Ludwigs-Universität Freiburg, Freiburg im Breisgau,
Germany. Available online: https://freidok.uni-freiburg.de/fedora/objects/freidok:
9664/datastreams/FILE1/content.

2 Options 39
Wong, W .-K., Thompson, H. E., & T eh, K. (2011). Was There Abnormal T rading
in the S&P 500 Index Options Prior to the September 11 Attacks? Multinational
Finance Journal, 15(3/4), 1–46.
Wystup, U. (2017). FX Options and Structured Products (2nd ed.). Chichester: Wiley.
Yim, H. L., Lee, S. H., Yoo, S. K., & Kim, J. J. (2011). A Zero-Cost Collar Option
Applied to Materials Procurement Contracts to Reduce Price Fluctuation Risks
in Construction. International Journal of Social, Behavioral, Educational, Economic,
Business and Industrial Engineering , 5(12), 1769–1774.
Youbi, F ., Pindza, E., & Maré, E. (2017). A Comparative Study of Spectral Methods
for Valuing Financial Options. Applied Mathematics & Information Sciences , 11(3),
939–950.
Zhang, C. (2015). Using Excel’s Data T able and Chart T ools Effectively in Finance
Courses. Journal of Accounting and Finance , 15(7), 79–93.
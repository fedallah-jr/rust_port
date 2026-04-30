# Appendix A: R Source Code for Backtesting

Appendix A: R Source Code for Backtesting
In this appendix we give the R (R Package for Statistical Computing, http://
www.r-project.org) source code for backtesting intraday strategies, where the
position is established at the open and liquidated at the close of the same
day. The sole purpose of this code is to illustrate some simple tricks for doing
out-of-sample backtesting. In particular, this code does not deal with the sur-
vivorship bias in any way,
1 albeit for this kind of strategies—precisely because
these are intraday strategies—the survivorship bias is not detrimental (see, e.g.,
Kakushadze 2015b).2
The main function (which internally calls some subfunctions) is
qrm.backtest() with the following inputs: (i) days is the lookback; (ii)
d.r is used for computing risk, both as the length of the moving standard
deviation tr (computed internally over d.r-day moving windows) as well
as the lookback for computing the risk model (and, if applicable, a statistical
industry classiﬁcation)—see below; (iii) d.addv is used as the lookback for the
average daily dollar volume addv, which is computed internally; (iv) n.addv
i st h en u m b e ro ft o pt i c k e r sb yaddv used as the trading universe, which is
recomputed every d.r days; (v) inv.lvl is the total investment level (long
plus short, and the strategy is dollar-neutral); (vi) bnds controls the position
bounds (which are the same in this strategy as the trading bounds), i.e., the
1I.e., simply put, it does not account for the fact that in the past there were tickers that are no longer there
at present, be it due to bankruptcies, mergers, acquisitions, etc. Instead, the input data is taken for the
tickers that exist on a given day by looking back, say, some number of years.
2For some literature related to the survivorship bias, which is important for longer-horizon strategies, see,
e.g., Amin and Kat ( 2003), Brown et al. ( 1992), Bu and Lacey ( 2007), Carhart et al. ( 2002), Davis ( 1996),
Elton et al. ( 1996b), Garcia and Gould ( 1993).
© The Editor(s) (if applicable) and The Author(s), under exclusive license
to Springer Nature Switzerland AG, part of Springer Nature 2018
Z. Kakushadze and J. A. Serur, 151 T rading Strategies,
https://doi.org/10.1007/978-3-030-02792-6
275

276 Appendix A: R Source Code for Backtesting
dollar holdings Hi for each stock are bounded via ( Bi are the bnds elements,
which can be uniform)
|Hi |≤ Bi Ai (A.1)
where i = 1,..., N labels the stocks in the trading universe, and Ai are the
corresponding elements of addv; (vii) incl.cost is a Boolean for including
linear trading costs, which are modeled as follows. 3 For the stock labeled by i ,
let Ei be its expected return, and wi be its weight in the portfolio. The source
code below determines wi via (mean-variance) optimization (with bounds).
For the stock labeled by i , let the linear trading cost per dollar traded be
τi . Including such costs in portfolio optimization amounts to replacing the
expected return of the portfolio
E port =
N∑
i =1
Ei wi (A.2)
by
E port =
N∑
i =1
[Ei wi − τi |wi |] (A.3)
A complete algorithm for including linear trading costs in mean-variance opti-
mization is given in, e.g., Kakushadze ( 2015b). However, for our purposes here
the following simple “hack” sufﬁces. We can deﬁne the effective return
E ef f
i = sign(Ei ) max(|Ei |− τi ,0)( A.4)
and simply set
E port =
N∑
i =1
E ef f
i wi (A.5)
I.e., if the magnitude for the expected return for a given stock is less than the
expected cost to be incurred, we set the expected return to zero, otherwise we
reduce said magnitude by said cost. This way we can avoid a nontrivial iterative
procedure (see, e.g., Kakushadze 2015b), albeit this is only an approximation.
3Here we closely follow the discussion in Sect. 3.1 of Kakushadze and Yu ( 2018b).

Appendix A: R Source Code for Backtesting 277
So, what should we use as τi in ( A.4)? The model of Almgren et al. ( 2005)
is reasonable for our purposes here. Let Hi be the dollar amount traded for the
stock labeled by i . Then for the linear trading costs we have
Ti = ζσ i
|Hi |
Ai
(A.6)
where σi is the historical volatility, Ai is the average daily dollar volume
(ADDV), and ζ is an overall normalization constant we need to ﬁx. How-
ever, above we work with weights wi , not traded dollar amounts Hi .I no u r
case of a purely intraday trading strategy discussed above, they are related sim-
ply via Hi = I wi ,w h e r eI is the total investment level (i.e., the total absolute
dollar holdings of the portfolio after establishing it). Therefore, we have (note
that Ti = τi |Hi |= τi I |wi |)
τi = ζ σi
Ai
(A.7)
We will ﬁx the overall normalization ζ via the following heuristic. We will
(conservatively) assume that the average linear trading cost per dollar traded
is 10 bps (1 bps = 1 basis point = 1/100 of 1%), 4 i.e., mean (τi ) = 10−3 and
ζ = 10−3/mean(σi /Ai ).
Next, internally the code sources price and volume data by reading it from
tab-delimited ﬁles 5 nrm.ret.txt (overnight return internally referred to as
ret—see below), nrm.open.txt (daily raw, unadjusted open price, inter-
nally referred to as open), nrm.close.txt (daily raw, unadjusted close
price, internally referred to as close), nrm.vol.txt (daily raw, unadjusted
volume, internally referred to as vol), nrm.prc.txt (daily close price fully
adjusted for all splits and dividends, internally referred to as prc). The rows
of ret, open, close, vol and prc correspond to the N tickers (index i ).
Let trading days be labeled by t = 0,1,2,..., T ,w h e r e t = 0 is the most
recent day. Then the columns of open, close, vol and prc correspond to
the trading days t = 1,2,..., T , i.e., the value of t is the same as the value
of the column index. On the other hand, the columns of ret correspond to
the overnight close-to-open returns from the trading day t to the trading day
t −1. I.e., the ﬁrst column of ret corresponds to the overnight close-to-open
return from the trading day t = 1 to the trading day t = 0. Furthermore, ret,
4This amounts to assuming that, to establish an equally-weighted portfolio, it costs 10 bps.
5This speciﬁc code does not use high, low, VWAP (volume-weighted average price), intraday (e.g., minute-
by-minute) prices, etc. However, it is straightforward to modify it such that it does.

278 Appendix A: R Source Code for Backtesting
call it Ri (t ),w h e r et = 1,2,..., T labels the columns of ret,i sc o m p u t e d
as follows:
Ri (t ) = ln
(
P AO
i (t − 1)
P AC
i (t )
)
(A.8)
P AO
i (t ) = γad j
i (t ) P O
i (t )( A.9)
γad j
i (t ) = P AC
i (t )
PC
i (t ) (A.10)
Here: P O
i (t ) is the raw open price (which is the corresponding element of
open for t = 1,2,..., T ); PC
i (t ) is the raw close price (which is the corre-
sponding element of close for t = 1,2,..., T ); P AC
i (t ) is the fully adjusted
close price (which is the corresponding element of prc for t = 1,2,..., T );
γad j
i (t )is the adjustment factor, which is used for computing the fully adjusted
open price P AO
i (t );s o Ri (t ) is the overnight, close-to-open return based on
fully adjusted prices. Note that the t = 0 prices required for computing Ri (1)
are not part of the matrices open, close and prc. Also, the code internally
assumes that the matrices ret, open, close, vol and prc are all aligned,
i.e., all tickers and dates are the same and in the same order in each of the
5ﬁ l e s nrm.ret.txt (note the labeling of the returns described above),
nrm.open.txt, nrm.close.txt, nrm.vol.txt and nrm.prc.txt.
The ordering of the tickers in these ﬁles is immaterial, so long as it is the
same in all 5 ﬁles as the code is oblivious to this ordering. However, the
dates must be ordered in the descending order, i.e., the ﬁrst column corre-
sponds to the most recent date, the second column corresponds to the date
before it, etc. (here “date” corresponds to a trading day). Finally, note that
the internal function read.x() reads these ﬁles with the parameter value
as.is = T . This means that these ﬁles are in the “R-ready” tab-delimited
format, with N + 1 tab-delimited lines. The lines 2 through N + 1 have
T + 1 elements each, the ﬁrst element being a ticker symbol (so the N ticker
symbols comprise dimnames(·)[[1]] of the corresponding matrix, e.g.,
open for the open prices), and the other T elements being the T values
(e.g., P O
i (t ), t = 1,..., T , for the open prices). However, the ﬁrst line has
only T elements, which are the labels of the trading days (so these comprise
dimnames(·)[[2]] of the corresponding matrix, e.g., open for the open
prices). Internal functions that use this input data, such as calc.mv.avg()
(which computes simple moving averages) and calc.mv.sd() (which com-
putes simple moving standard deviations) are simple and self-explanatory.

Appendix A: R Source Code for Backtesting 279
As mentioned above, the input parameter d.r is used for recomputing the
trading universe every d.r trading days and also recomputing the risk models
(see below) every d.r trading days.These computations are done 100% out-of-
sample, i.e., the data used in these computations is 100% in the past w.r.t. to the
trading day on which the resultant quantities are used for (simulated) trading.
This is accomplished in part by using the internal function calc.ix().N o t e
that the input data described above is structured and further used in such a way
that the backtests are 100% out-of-sample. Here two conceptually different
aspects must be distinguished. Thus, we have the expected returns and “the
rest”, the latter—which can be loosely referred to as “risk management”—being
the universe selection, the risk model computation, etc., i.e., the machinery that
gets us from the expected returns to the desired holdings (that is, the strategy
positions). The risk management part must be 100% out-of-sample. In real life
the expected returns are also 100% out-of-sample. However, in backtesting,
while the expected returns cannot under any circumstances look into the future,
they can sometimes be “borderline in-sample”. Thus, consider a strategy that
today trades on the overnight yesterday’s-close-to-today’s-open return. If we
assume that the positions are established based on this return sometime after
the open, then the backtest is out-of-sample by the “delay” time between the
open and when the position is established. However, if we assume that the
position is established at the open, then this is the so-called “delay-0” strategy,
and the backtest is “borderline in-sample” in the sense that in real life the orders
would have to be sent with some, albeit possibly small, delay, but could never
be executed exactly at the open. In this sense it still makes sense to backtest such
a strategy to measure the strength of the signal. What would make no sense and
should never be done is to run an outright in-sample backtest that looks into
the future. E.g., using today’s closing prices for computing expected returns for
trading at today’s open would be grossly in-sample. On the other hand, using
yesterday’s prices to trade at today’s open is the so-called “delay-1” strategy,
which is basically 1 day out-of-sample (and, not surprisingly, is expected to
backtest much worse than a delay-0 strategy). The code gives examples of
both delay-0 (mean-reversion) and delay-1 (momentum) strategies (see the
comments
DELAY-0 and DELAY-1 in the code).
The code internally computes the desired holdings via optimization. The
optimizer function (which incorporates bounds and linear constraints such
as dollar-neutrality) bopt.calc.opt() is given in Kakushadze ( 2015e).
One of its inputs is the inverse model covariance matrix for the stocks.
This matrix is computed internally via functions such as qrm.cov.pc()
and qrm.erank.pc(), which are given in and utilize the statistical risk
model construction of Kakushadze and Yu ( 2017a), or qrm.gen.het(),

280 Appendix A: R Source Code for Backtesting
which is given in and utilizes the heterotic risk model construction of
Kakushadze and Yu ( 2016a). The latter requires a multilevel binary indus-
try classiﬁcation. The code below builds such a classiﬁcation via the function
qrm.stat.ind.class.all(), which is given in and utilizes the statistical
industry classiﬁcation construction of Kakushadze and Yu ( 2016b). However,
the code can be straightforwardly modiﬁed to utilize a fundamental industry
classiﬁcation, such as GICS (Global Industry Classiﬁcation Standard), BICS
(Bloomberg Industry Classiﬁcation System), SIC (Standard Industrial Classiﬁ-
cation), etc. One issue with this is that practically it is difﬁcult to do this 100%
out-of-sample. However, “in-sampleness” of a fundamental industry classiﬁ-
cation—which is relatively stable—typically does not pose a serious issue in
such backtests as stocks rarely jump industries. Furthermore, note that the
aforesaid “external” functions have various other parameters (which are set to
their implicit default values in the code below), which can be modiﬁed (see
the references above that provide the aforesaid functions).
Finally, the code internally computes the desired holdings and various per-
formance characteristics such as the total P&L over the backtesting period,
annualized return, annualized Sharpe ratio, and cents-per-share. These and
other quantities computed internally can be returned (e.g., via environments
or lists), dumped into ﬁles, printed on-screen, etc. The code is straightforward
and can be tweaked depending on the user’s speciﬁc needs/strategies. Its pur-
pose is illustrative/pedagogical.
qrm.backtest <- function (days = 252 * 5, d.r = 21,
d.addv = 21, n.addv = 2000, inv.lvl = 2e+07, bnds = .01,
incl.cost = F)
{
calc.ix <- function(i, d, d.r)
{
k 1< -d-i
k1 <- trunc(k1 / d.r)
i x< -d-k 1*d . r
return(ix)
}
calc.mv.avg <- function(x, days, d.r)
{
y <- matrix(0, nrow(x), days)
for(i in 1:days)
y[, i] <- rowMeans(x[, i:(i + d.r - 1)])

Appendix A: R Source Code for Backtesting 281
return(y)
}
calc.mv.sd <- function(x, days, d.r)
{
y <- matrix(0, nrow(x), days)
for(i in 1:days)
y[, i] <- apply(x[, i:(i + d.r - 1)], 1, sd)
return(y)
}
read.x <- function(file)
{
x <- read.delim(file, as.is = T)
x <- as.matrix(x)
mode(x) <- "numeric"
return(x)
}
calc.sharpe <- function (pnl, inv.lvl)
{
print(sum(pnl, na.rm = T))
print(mean(pnl, na.rm = T) * 252 / inv.lvl * 100)
print(mean(pnl, na.rm = T) / sd(pnl, na.rm = T)
* sqrt(252))
}
ret <- read.x("nrm.ret.txt")
open <- read.x("nrm.open.txt")
close <- read.x("nrm.close.txt")
vol <- read.x("nrm.vol.txt")
prc <- read.x("nrm.prc.txt")
addv <- calc.mv.avg(vol * close, days, d.addv)
ret.close <- log(prc[, -ncol(prc)]/prc[, -1])
tr <- calc.mv.sd(ret.close, days, d.r)
ret <- ret[, 1:days]

282 Appendix A: R Source Code for Backtesting
prc <- prc[, 1:days]
close <- close[, 1:days]
open <- open[, 1:days]
close1 <- cbind(close[, 1], close[, -ncol(close)])
open1 <- cbind(close[, 1], open[, -ncol(open)])
pnl <- matrix(0, nrow(ret), ncol(ret))
des.hold <- matrix(0, nrow(ret), ncol(ret))
for(i in 1:ncol(ret))
{
ix <- calc.ix(i, ncol(ret), d.r)
if(i == 1)
prev.ix <- 0
if(ix != prev.ix)
{
liq <- addv[, ix]
x <- sort(liq)
x <- x[length(x):1]
take <- liq >= x[n.addv]
r1 <- ret.close[take, (ix:(ix + d.r - 1))]
### ind.list <- qrm.stat.ind.class.all(r1,
### c(100, 30, 10), iter.max = 100)
### rr <- qrm.gen.het(r1, ind.list)
rr <- qrm.cov.pc(r1)
### rr <- qrm.erank.pc(r1)
cov.mat <- rr$inv.cov
prev.ix <- ix
}
w.int <- rep(1, sum(take))
ret.opt <- ret ### DELAY-0 MEAN-REVERSION
### ret.opt <- -log(close/open) ### DELAY-1 MOMENTUM

Appendix A: R Source Code for Backtesting 283
if(incl.cost)
{
lin.cost <- tr[take, i] / addv[take, i]
lin.cost <- 1e-3 * lin.cost / mean(lin.cost)
}
else
lin.cost <- 0
ret.lin.cost <- ret.opt[take, i]
ret.lin.cost <- sign(ret.lin.cost) *
pmax(abs(ret.lin.cost) - lin.cost, 0)
des.hold[take, i] <- as.vector(bopt.calc.opt
(ret.lin.cost, w.int,
cov.mat, bnds * liq[take]/inv.lvl, -bnds
* liq[take]/inv.lvl))
des.hold[take, i] <- -des.hold[take, i] *
inv.lvl / sum(abs(des.hold[take, i]))
pnl[take, i] <- des.hold[take, i] *
(close1[take, i]/open1[take, i] - 1)
pnl[take, i] <- pnl[take, i] - abs(des.hold[take,
i]) * lin.cost
}
des.hold <- des.hold[, -1]
pnl <- pnl[, -1]
pnl <- colSums(pnl)
calc.sharpe(pnl, inv.lvl)
trd.vol <- 2 * sum(abs(des.hold/open1[, -1]))
cps <- 100 * sum(pnl) / trd.vol
print(cps)
}
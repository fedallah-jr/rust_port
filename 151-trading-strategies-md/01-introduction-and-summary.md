# Chapter 1: Introduction and Summary

1
Introduction and Summary
A trading strategy can be deﬁned as a set of instructions to achieve certain asset
holdings by some predeﬁned times t1, t2,... , which holdings can (but need
not) be null at one or more of these times. In many cases, the main objective
of a trading strategy is to make a proﬁt, i.e., to generate a positive return on
its investment. However, some viable trading strategies are not always outright
proﬁtable as stand-alone strategies. E.g., a hedging strategy can be a part of a
bigger plan, which itself can but need not be a trading strategy. Thus, an airline
hedging against rising fuel costs with commodity futures is a trading strategy,
which is a risk management step in executing the airline’s business strategy of
generating proﬁts through its services.
In the case of trading strategies that are intended to be outright proﬁtable
as stand-alone strategies, one may argue that the phrase “buy low, sell high”
captures their essence. However, this viewpoint is somewhat superﬂuous and,
while it applies to trading strategies that buy and sell a single asset (e.g., a
single stock), it would exclude a whole host of viable strategies that do not
work quite like that. E.g., a trading strategy that uses a hedging sub-strategy
for risk management may not always “buy low, sell high” when it comes to a
particular asset in its portfolio. This is because hedging risk—or, essentially,
transferring some risk to other market participants—is not free, and often a
trader will pay a premium for hedging some risks in a trading strategy to achieve
its objectives. Another example would be the so-called statistical arbitrage,
wherein the trading portfolio can consist of, e.g., thousands of stocks and
proﬁtability is typically not achieved by buying low and selling high each stock
or even any discernable groups of stocks, but statistically, across all stocks, with
some trades making money and some losing it. It gets complicated quickly.
© The Author(s) 2018
Z. Kakushadze and J. A. Serur, 151 T rading Strategies,
https://doi.org/10.1007/978-3-030-02792-6_1
1

2 Z. Kakushadze and J. A. Serur
The purpose of these notes is to collect a variety of trading strategies in
the context of ﬁnance (as opposed to trading baseball cards, classic cars, etc.)
across essentially all (or at least most frequently encountered) asset classes.
Here we deliberately use the term “asset class” somewhat loosely and include
what can be referred to as “asset sub-classes”. Thus, a narrower deﬁnition
would include stocks, bonds, cash, currencies, real estate, commodities, and
infrastructure. However, this deﬁnition would be too narrow for our purposes
here. We also consider: derivatives such as options and futures; exchange-
traded funds (ETFs); indexes (which are usually traded through vehicles
such as ETFs and futures); volatility, which can be treated as an asset class
(and traded via, among other things, exchange-traded notes); structured assets
(such as collateralized debt obligations and mortgage-backed securities); con-
vertible bonds (which represent a hybrid between bonds and stocks); distressed
assets (which are not a separate asset class per se, but the corresponding trading
strategies are rather distinct); cryptocurrencies; miscellaneous assets such as
weather and energy (derivatives); and also trading strategies such as tax arbi-
trage and global macro (which use some assets mentioned above as tradables).
Some strategies are relatively simple and can be described in words, while many
(in fact, most) require a much more detailed mathematical description, which
we provide formulaically.
It is important to bear in mind that, unlike the laws of nature (physics),
which (apparently) are set in stone and do not change in time, ﬁnancial markets
are man-made and change essentially continuously, and at times quite dramat-
ically. One of the consequences of this transiency is that trading strategies
that may have worked well for some time, may die, sometimes quite abruptly.
E.g., when the New York Stock Exchange (NYSE) started switching away
from its human-operated “specialist” system to electronic trading beginning
late 2006,
1 many statistical arbitrage strategies that were proﬁtable for years
prior to that, pretty much died overnight as volatility increased and what used
to do the trick before no longer did. Eventually, the market was ﬂooded with
high-frequency trading (HFT) 2 strategies further diminishing proﬁt margins
of many “good old” trading strategies and killing them.
However, technological advances gave rise to new types of trading, includ-
ing ubiquitous trading strategies based on data mining and machine learning,
which seek to identify—typically quite ephemeral—signals or trends by ana-
lyzing large volumes of diverse types of data. Many of these trading signals are
1NYSE ﬁrst started with its “Hybrid Market” (see, e.g., Hendershott and Moulton 2011). However, the
writing had been on the wall for the ultimate demise of the specialist system for quite some time. For a
timeline, see, e.g., Pisani ( 2010).
2See, e.g., Aldridge ( 2013), Lewis ( 2014).

1 Introduction and Summary 3
so faint that they cannot be traded on their own, so one combines thousands,
in fact, tens or even hundreds of thousands if not millions of such signals
with nontrivial weights to amplify and enhance the overall signal such that it
becomes tradable on its own and proﬁtable after trading costs and slippage,
including that inﬂicted by HFT. 3
Considering the intrinsically ephemeral nature of the ﬁnancial markets and
trading strategies designed to make a proﬁt therefrom, the purpose of these
notes is not to convey to the reader how to make money using any trading
strategy but simply to provide information on and give some ﬂavor of what
kind of trading strategies people have considered across a broad cross-section of
asset classes and trading styles. In light of the foregoing, we make the following
DISCLAIMER: Any information or opinions provided herein are for informa-
tional purposes only and are not intended, and shall not be construed, as an invest-
ment, legal, tax or any other such advice, or an offer, solicitation, recommendation
or endorsement of any trading strategy , security , product or service. For further
legal disclaimers, see Appendix B hereof.
We hope these notes will be useful to academics, practitioners, students and
aspiring researchers/traders for years to come. These notes intentionally—not
to duplicate prior literature and to avoid this manuscript spanning thousands of
pages—do not contain any numeric simulations, backtests, empirical studies,
etc. However, we do provide an eclectic cornucopia of references, including
those with detailed empirical analyses. Our purpose here is to describe, in many
cases in sizable detail, various trading strategies. Also, Appendix A provides
source code for illustrating out-of-sample backtesting (see Appendix B for
legalese).
4 So, we hope you enjoy!
References
Aldridge, I. (2013). High-Frequency T rading: A Practical Guide to Algorithmic Strategies
and T rading Systems (2nd ed.). Hoboken, NJ: Wiley.
Hendershott, T ., & Moulton, P . C. (2011). Automation, Speed, and Stock Market
Quality: The NYSE’s Hybrid. Journal of Financial Markets , 14 (4), 568–604.
Kakushadze, Z., & T ulchinsky, I. (2016). Performance v. T urnover: A Story by 4,000
Alphas. Journal of Investment Strategies , 5 (2), 75–89. Available online: http://ssrn.
com/abstract=2657603.
Kakushadze, Z., & Yu, W . (2017). How to Combine a Billion Alphas. Journal of Asset
Management, 18 (1), 64–80. Available online: https://ssrn.com/abstract=2739219.
3See, e.g., Kakushadze and T ulchinsky ( 2016), Kakushadze and Yu ( 2017).
4The code in Appendix A is not written to be “fancy” or optimized for speed or otherwise.

4 Z. Kakushadze and J. A. Serur
Lewis, M. (2014). Flash Boys: A W all Street Revolt . Ne w Yo r k , N Y: W. W. No r t o n .
Pisani, B. (2010, September 13). Man Vs. Machine: How Stock T rading Got So
Complex. CNBC. Available online: https://www.cnbc.com/id/38978686.
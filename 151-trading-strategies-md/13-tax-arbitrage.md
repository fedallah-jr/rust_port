# Chapter 13: Tax Arbitrage

13
T ax Arbitrage
13.1 Strategy: Municipal Bond T ax Arbitrage
This strategy is one of the most common and simple forms of tax arbitrage.
It amounts to borrowing money and buying tax-exempt municipal bonds. 1
The strategy return is given by
R = rlong − rshort (1 − τ) (13.1)
Here: rlong is the interest rate of the bought municipal bonds, rshort is the
interest rate of the loan, and τ is the corporate tax rate. This strategy is attrac-
tive to companies in jurisdictions where tax rules allow them to buy tax-exempt
municipal bonds and deduct interest expenses from their taxable income
(a.k.a. “tax shield”).
13.2 Strategy: Cross-Border T ax Arbitrage
The U.S. double-taxes corporate income. The corporate income is ﬁrst taxed
at the corporate level. Then, it is taxed again when dividends are received by
the shareholders. In some other countries the taxation systems are designed to
relieve the tax burden, e.g., by not taxing dividends (as, e.g., in Singapore), or
by giving shareholders tax credits attached to dividend payments (as, e.g., in
1For some literature on municipal bond tax arbitrage and related topics, see, e.g., Ang et al. ( 2017), Buser
and Hess ( 1986), Chalmers ( 1998), Erickson et al. ( 2003), Heaton ( 1988), Kochin and Parks ( 1988),
Longstaff ( 2011), Miller ( 1977), Poterba et al. ( 1986, 1989), Skelton ( 1983), T rzcinka (1982), Yawitz et
al. ( 1985).
© The Author(s) 2018
Z. Kakushadze and J. A. Serur, 151 T rading Strategies,
https://doi.org/10.1007/978-3-030-02792-6_13
199

200 Z. Kakushadze and J. A. Serur
Australia). In the case where this “dividend imputation” corporate tax system
gives the full tax credit to shareholders, it can be schematically described as
follows (see, e.g., McDonald 2001)2:
⎧
⎪⎪
⎪
⎪
⎪
⎪
⎪
⎪
⎪⎪⎪
⎨
⎪⎪⎪⎪
⎪
⎪
⎪
⎪
⎪
⎪
⎪
⎩
Corporate tax rate = τ
c
Cash dividend paid = D
Dividend tax credit = C = D τc
1−τc
T axable income = It = D + C = D
1−τc
Personal tax rate = τp
Personal income tax = T = It τp
Dividend income after credit and tax = I = D + C − T = D 1−τp
1−τc
(13.2)
So, if the corporate income is P and the corporation pays all its income after
taxes as dividends, i.e., if D = P (1 − τc),t h e n I = P
(
1 − τp
)
,s ot h e r ei s
no double-taxation. 3
While in countries with imputation systems domestic investors enjoy tax
credits, generally foreign investors do not. If there were no tax credits, the
price drop between cum-dividend and ex-dividend 4 is expected to reﬂect the
dividend. In the presence of tax credits, the drop is expected to be higher: if it
fully reﬂects the tax credit, then it is D (1 + κ),w h e r eκ is the tax credit rate.
(In the above nomenclature, 1 + κ = 1/(1 + τc)). So, a foreign investor is
effectively penalized for holding the stock. T o avoid this, the foreign investor
can sell the stock cum-dividend and buy it back ex-dividend. 5 Alternatively,
the foreign investor can loan the stock to a domestic investor cum-dividend
and receive the stock back ex-dividend along with (some preset portion of )
the tax credit—assuming no restrictions on such cross-border tax arbitrage.
A swap agreement would also achieve the same result.
6
2However, there can be limitations on the tax credit and other subtleties present depending on the
jurisdiction, various circumstances, etc.
3In contrast, in the double-taxation system we would instead have: D = P (1 − τc ), It = D, T = It τp ,
I = It − T = P (1 − τc )
(
1 − τp
)
.
4Cum-dividend means the stock buyer is entitled to receive a dividend that has been declared but not
paid. Ex-dividend means the stock seller is entitled to the dividend, not the buyer.
5Assuming transaction costs are not prohibitively high.
6For some literature on cross-border tax arbitrage and related topics, see, e.g., Allen and Michaely ( 1995),
Amihud and Murgia ( 1997), Bellamy ( 1994), Booth ( 1987), Booth and Johnston ( 1984), Brown and
Clarke ( 1993), Bundgaard ( 2013), Callaghan and Barry ( 2003), Christoffersen et al. ( 2003, 2005), Eun
and Sabherwal ( 2003), Green and Rydqvist ( 1999), Harris et al. ( 2001), Lakonishok and Vermaelen
(1986), Lasfer ( 1995), Lessambo ( 2016), McDonald ( 2001), Monkhouse ( 1993), Shaviro ( 2002), Wells
(2016), Wood ( 1997).

13 T ax Arbitrage 201
Strategy: Cross-Border T ax Arbitrage with Options
Absent a tax credit, there is a theoretical upper bound on the value of an
American put option (see, e.g., Hull 2012):
Vput (K , T ) ≤ Vcall (K , T ) − S0 + K + D (13.3)
Here: Vput (Vcall ) is the price of the put (call) option at time t = 0; K is the
strike price; S0 is the stock price at t = 0; T is the time to maturity; and D
is the present value of the dividends during the life of the option. Put options
are optimally exercised ex-dividend. Therefore, in the presence of a tax credit,
it is expected that put prices should reﬂect the tax credit, i.e., they should be
higher than in the absence of the tax credit (see, e.g., McDonald 2001). So the
foreign investor can sell the stock cum-dividend (at price S0) and write a deep
ITM put option, whose value close to expiration approximately is (here κ is
the tax credit rate deﬁned above)
Vput (K , T ) = K − [S0 − D (1 + κ)] (13.4)
The P&L, once the put is exercised ex-dividend at the strike price K ,i st h e
same as with the stock loan/swap strategy discussed above:
P&L = S0 + Vput (K , T ) − K = D (1 + κ) (13.5)
References
Allen, F ., & Michaely, R. (1995). Dividend Policy. In R. A. Jarrow, V . Maksimovic,
& W . T . Ziemba (Eds.),Handbooks in Operations Research and Management Science
(Vol. 9, Chapter 25, pp. 793–837). Amsterdam, The Netherlands: Elsevier.
Amihud, Y., & Murgia, M. (1997). Dividends, T axes, and Signaling: Evidence from
Germany. Journal of Finance , 52 (1), 397–408.
Ang, A., Green, R. C., Longstaff, F . A., & Xing, Y. (2017). Advance Refundings of
Municipal Bonds. Journal of Finance , 72 (4), 1645–1682.
Bellamy, D. E. (1994). Evidence of Imputation Clienteles in the Australian Equity
Market. Asia Paciﬁc Journal of Management , 11(2), 275–287.
Booth, L. D. (1987). The Dividend T ax Credit and Canadian Ownership Objectives.
Canadian Journal of Economics , 20 (2), 321–339.
Booth, L. D., & Johnston, D. J. (1984). The Ex-dividend Day Behavior of Canadian
Stock Prices: T ax Changes and Clientele Effects. Journal of Finance , 39 (2), 457–
476.

202 Z. Kakushadze and J. A. Serur
Brown, P ., & Clarke, A. (1993). The Ex-dividend Day Behaviour of Australian Share
Prices Before and After Dividend Imputation. Australian Journal of Management ,
18(1), 1–40.
Bundgaard, J. (2013). Coordination Rules as a Weapon in the War Against Cross-
Border T ax Arbitrage—The Case of Hybrid Entities and Hybrid Financial Instru-
ments. Bulletin for International T axation, April/May, 2013 , 200–204.
Buser, S. A., & Hess, P . J. (1986). Empirical Determinants of the Relative Yields
on T axable and T ax-Exempt Securities. Journal of Financial Economics , 17 (2),
335–355.
Callaghan, S. R., & Barry, C. B. (2003). T ax-Induced T rading of Equity Securities:
Evidence from the ADR Market. Journal of Finance , 58(4), 1583–1611.
Chalmers, J. M. R. (1998). Default Risk Cannot Explain the Muni Puzzle: Evidence
from Municipal Bonds That Are Secured by U.S. T reasury Obligations. Review of
Financial Studies , 11(2), 281–308.
Christoffersen, S. E. K., Géczy, C. C., Musto, D. K., & Reed, A. V . (2005). Crossbor-
der Dividend T axation and the Preferences of T axable and Nontaxable Investors:
Evidence From Canada. Journal of Financial Economics , 78(1), 121–144.
Christoffersen, S. E. K., Reed, A. V ., Géczy, C. C., & Musto, D. K. (2003). The
Limits to Dividend Arbitrage: Implications for Cross Border Investment (Working
Paper). Available online: https://ssrn.com/abstract=413867.
Erickson, M., Goolsbee, A., & Maydew, E. (2003). How Prevalent is T ax Arbitrage?
Evidence from the Market for Municipal Bonds. National T ax Journal , 56 (1),
259–270.
Eun, C. S., & Sabherwal, S. (2003). Cross-Border Listings and Price Discovery: Evi-
dence from U.S. Listed Canadian Stocks. Journal of Finance , 58(2), 549–575.
Green, R. C., & Rydqvist, K. (1999). Ex-day Behavior with Dividend Preference and
Limitations to Short-T erm Arbitrage: The Case of Swedish Lottery Bonds. Journal
of Financial Economics , 53(2), 145–187.
Harris, T . S., Hubbard, R. G., & Kemsley, D. (2001). The Share Price Effects of
Dividend T axes and T ax Imputation Credits. Journal of Public Economics , 79 (3),
569–596.
Heaton, H. (1988). On the Possible T ax-Driven Arbitrage Opportunities in the New
Municipal Bond Futures Contract. Journal of Futures Markets , 8(3), 291–302.
Hull, J. C. (2012). Options, Futures and Other Derivatives . Upper Saddle River, NJ:
Prentice Hall.
Kochin, L., & Parks, R. (1988). Was the T ax-Exempt Bond Market Inefﬁcient or Were
Future Expected T ax Rates Negative? Journal of Finance , 43(4), 913–931.
Lakonishok, J., & Vermaelen, T . (1986). T ax-Induced T rading Around the Ex-day.
Journal of Financial Economics , 16 (3), 287–319.
Lasfer, M. A. (1995). Ex-day Behavior: T ax or Short-T erm T rading Effects. Journal of
Finance, 50 (3), 875–897.
Lessambo, F . I. (2016). International Aspects of the US T axation System .N e wY o r k ,N Y :
Palgrave Macmillan.

13 T ax Arbitrage 203
Longstaff, F . A. (2011). Municipal Debt and Marginal T ax Rates: Is There a T ax
Premium in Asset Prices? Journal of Finance , 66 (3), 721–751.
McDonald, R. L. (2001). Cross-Border Investing with T ax Arbitrage: The Case of
German Dividend T ax Credits. Review of Financial Studies , 14 (3), 617–657.
Miller, M. H. (1977). Debt and T axes. Journal of Finance , 32 (2), 261–275.
Monkhouse, P . H. L. (1993). The Cost of Equity Under the Australian Dividend
Imputation T ax System. Accounting and Finance , 33(2), 1–18.
Poterba, J. (1986). Explaining the Yield Spread Between T axable and T ax Exempt
Bonds. In H. Rosen (Ed.), Studies in State and Local Public Finance (pp. 5–48).
Chicago, IL: University of Chicago Press.
Poterba, J. (1989). T ax Reform and the Market for T ax-Exempt Debt. Regional Science
and Urban Economics , 19 (3), 537–562.
Shaviro, D. (2002). Dynamic Strategies for Asset Allocation. Chicago Journal of Inter-
national Law , 3(2), 317–331.
Skelton, J. L. (1983). Banks, Firms and the Relative Pricing of T ax-Exempt and T axable
Bonds. Journal of Financial Economics , 12 (3), 343–355.
T rzcinka, C. (1982). The Pricing of T ax-Exempt Bonds and the Miller Hypothesis.
Journal of Finance , 37 (4), 907–923.
Wells, B. (2016). The Foreign T ax Credit War. Brigham Young University Law Review ,
6, 1895–1965.
Wood, J. (1997). A Simple Model for Pricing Imputation T ax Credits Under Australia’s
Dividend Imputation T ax System. Paciﬁc-Basin Finance Journal , 5 (4), 465–480.
Yawitz, J. B., Maloney, K. J., & Ederington, L. H. (1985). T axes, Default Risk, and
Yield Spreads. Journal of Finance , 40 (4), 1127–1140.
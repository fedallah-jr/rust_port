# Chapter 18: Cryptocurrencies

18
Cryptocurrencies
18.1 Generalities
Cryptocurrencies, such as Bitcoin (BTC), Ethereum (ETH), etc., unlike
traditional ﬁat currencies (USD, EUR, etc.), are decentralized digital currencies
based on open-source peer-to-peer (P2P) internet protocols. Cryptocurrencies
such as BTC and ETH use the blockchain technology (Nakamoto 2008).1
T otal market capitalization of cryptocurrencies is measured in hundreds of
billions of dollars. 2 Many investors are attracted to cryptocurrencies as specu-
lative buy-and-hold assets. Thus, some view them as diversiﬁers due to their low
correlation with traditional assets. Others perceive them as the future of money.
Some investors simply want to make a quick buck on a speculative bubble.
Etc.
3 Be it as it may, unlike, e.g., stocks, there are no evident “fundamen-
tals” for cryptoassets based on which one could build “fundamental” trading
1Blockchain is a distributed ledger, which keeps a record of all transactions. It is a sequential chain of
blocks, which are linked using cryptography and time-stamping, containing transaction records. No block
can be altered retroactively without altering all subsequent blocks, which renders blockchain resistant to
data modiﬁcation by its very design. For a blockchain maintained by a large network as a distributed ledger
continuously updated on a large number of systems simultaneously, collusion of the network majority
would be required for a nefarious modiﬁcation of blockchain.
2Cryptocurrencies are highly volatile, so their market cap has substantial time variability.
3For some pertinent literature, see, e.g., Baek and Elbeck ( 2014), Bariviera et al. ( 2017), Bouoiyour et al.
(2015), Bouoiyour et al. ( 2016), Bouri et al. ( 2017a), Bouri et al. ( 2017b), Brandvold et al. ( 2015), Brière
et al. ( 2015), Cheah and Fry ( 2015), Cheung et al. ( 2015), Ciaian et al. ( 2015), Donier and Bouchaud
(2015), Dowd and Hutchinson ( 2015), Dyhrberg ( 2015), Dyhrberg ( 2016), Eisl et al. ( 2015), Fry and
Cheah ( 2016), Gajardo et al. ( 2018), Garcia and Schweitzer ( 2015), Garcia et al. ( 2014), Harvey ( 2014,
2016), Kim et al. ( 2016), Kristoufek ( 2015), Lee et al. ( 2018), Liew et al. ( 2018), Ortisi ( 2016), Van
Alstyne ( 2014), Wang and V ergne ( 2017), White ( 2015).
© The Author(s) 2018
Z. Kakushadze and J. A. Serur, 151 T rading Strategies,
https://doi.org/10.1007/978-3-030-02792-6_18
249

250 Z. Kakushadze and J. A. Serur
strategies (e.g., value-based strategies). So, cryptocurrency trading strategies
tend to rely on trend data mining via machine learning techniques.
18.2 Strategy: Artificial Neural Network (ANN)
This strategy uses ANN to forecast short-term movements of BTC based on
input technical indicators. In ANN we have an input layer, an output layer,
and some number of hidden layers. So, in this strategy the input layer is
built using technical indicators. 4 E.g., we can use (exponential) moving aver-
ages ((E)MAs), (exponential) moving standard deviations ((E)MSDs), relative
strength index (RSI), 5 etc. More concretely, we can construct the input layer
as follows (see, e.g., Nakano et al. 2018). Let P(t ) b et h eB T Cp r i c ea tt i m et ,
where t = 1,2,... is measured in some units (e.g., 15-minute intervals; also,
t = 1 is the most recent time). Let:
R(t ) = P(t )
P(t + 1) − 1 (18.1)
˜R(t, T1) = R(t ) − R(t, T1)( 18.2)
R(t, T1) = 1
T1
t +T1∑
t ′=t +1
R(t ′)( 18.3)
ˆR(t, T1) =
˜R(t, T1)
σ(t, T1) (18.4)
[σ(t, T1)]2 = 1
T1 − 1
t +T1∑
t ′=t +1
[˜R(t, T1)]2 (18.5)
So: R(t ) is the return from t + 1 to t ; R(t, T1) is the serial mean return from
t +T1 to t +1, i.e., over T1 periods, where T1 can be chosen to be long enough
to provide a reasonable estimate for the volatility (see below); ˜R(t, T1) is the
serially demeaned return; σ(t, T1) is the volatility computed from t + T1 to
t + 1;a n d ˆR(t, T1) is the normalized (serially demeaned) return. Below, for
notational simplicity we will omit the reference to the T1 parameter and will
use ˆR(t ) to denote the normalized returns.
4Thus, in spirit, it is somewhat similar to the single-stock KNN trading strategy discussed in Sect. 3.17,
which utilizes the k-nearest neighbor (KNN) algorithm (as opposed to ANN).
5T ypically, RSI> 0.7 (< 0.3) is interpreted as overbought (oversold). See, e.g., Wilder ( 1978).

18 Cryptocurrencies 251
Next, we can deﬁne EMAs, EMSDs and RSI as follows 6:
EMA(t,λ,τ) = 1 − λ
1 − λτ
t +τ∑
t ′=t +1
λt ′−t −1 ˆR(t ′)( 18.6)
[EMSD(t,λ,τ) ]2 = 1 − λ
λ − λτ
t +τ∑
t ′=t +1
λt ′−t −1 [ˆR(t ′) − EMA(t,λ,τ) ]2 (18.7)
RSI(t,τ) = ν+(t,τ)
ν+(t,τ) + ν−(t,τ) (18.8)
ν±(t,τ) =
t +τ∑
t ′=t +1
max(±ˆR(t ′),0)( 18.9)
Here: τ is the moving average length; λ is the exponential smoothing param-
eter.7
The input layer can then be deﬁned as consisting of, e.g., ˆR(t ),
EMA(t,λa ,τa ), EMSD (t,λa ,τa ),a n dR S I (t,τ ′
a′),w h e r e a = 1,..., m,
a′ = 1,..., m′.T h ev a l u e sτa can, e.g., be chosen to correspond to 30 min,
1 hr, 3 hrs, and 6 hrs (so m = 4; see fn. 7 for the values of λa ). The values τ′
a′
can, e.g., be chosen to correspond to 3 hrs, 6 hrs, and 12 hrs (so m′ = 3). There
is no magic bullet here. These values can be chosen based on out-of-sample
backtests keeping in mind, however, the ever-present danger of over-ﬁtting
various free parameters (see below), including τa , λa and τ′
a′.
The output layer can be constructed as follows. Let the objective be to
forecast which quantile the future normalized return will belong to. Let the
number of quantiles be K . Thus, for the values of t corresponding to the
training dataset D
train ,8 we have the normalized returns ˆR(t ), t ∈ Dtrain .L e t
the (K −1)quantile values of ˆR(t ), t ∈ Dtrain ,b e qα, α = 1,...,( K −1).F o r
each value of t , we can deﬁne the supervisory K -vectors Sα(t ), α = 1,..., K ,
as follows:
6Note that this can be done in more than one way.
7T o reduce the number of parameters, we can, e.g., take λ = (τ − 1)/(τ + 1).
8Ideally, when computing the quantiles, an appropriate number d1 of the values of t =
td ,td−1 ,..., td−d1 +1 , d =| Dtrain |, should be excluded to ensure that all the EMA, EMSD and RSI
values are computed using the required numbers of datapoints.

252 Z. Kakushadze and J. A. Serur
⎧
⎪⎪
⎪
⎨
⎪⎪
⎪
⎩
S
1(t ) = 1, ˆR(t ) ≤ q1
Sα(t ) = 1, qα−1 ≤ ˆR(t )< qα, 1 <α< K
SK (t ) = 1, qK −1 ≤ ˆR(t )
Sα(t ) = 0, otherwise
(18.10)
The output layer can then be a nonnegative K -vector pα(t ), whose elements
are interpreted as the probabilities of the future normalized return to be in the
α-th quantile. So, we have
K∑
α=1
pα(t ) = 1 (18.11)
The output layer is constructed from the input layer as some nonlinear function
thereof with some number of parameters to be determined via training. In
ANN we have L layers labeled by ℓ = 1,..., L,w h e r eℓ = 1 corresponds to
the input layer, and ℓ = L corresponds to the output layer. At each layer we
have N (ℓ) nodes and the corresponding N (ℓ) -vectors ⃗X (ℓ) with components
X (ℓ)
i (ℓ) , i (ℓ) = 1,..., N (ℓ) 9:
X (ℓ)
i (ℓ) = h(ℓ)
i (ℓ) ( ⃗Y (ℓ) ), ℓ = 2,..., L (18.12)
Y (ℓ)
i (ℓ) =
N (ℓ−1)
∑
j (ℓ−1)=1
A(ℓ)
i (ℓ) j (ℓ−1) X (ℓ−1)
j (ℓ−1) + B(ℓ)
i (ℓ) (18.13)
Here: ⃗Y (ℓ) is an N (ℓ) -vector with components Y (ℓ)
i (ℓ) , i (ℓ) = 1,..., N (ℓ) ;
X (1)
i (1) are the input data (for each value of t , i.e., ˆR(t ),E M A (t,λa ,τa ),
EMSD(t,λa ,τa ),a n dR S I(t,τ ′
a′) – see above); X (L)
i (L) are the output data pα(t )
(i.e., N (L) = K and the index i (L) is the same as α); the unknown parameters
A(ℓ)
i (ℓ) j (ℓ−1) (the so-called weights) and B(ℓ)
i (ℓ) (the so-called bias) are determined
via training (see below); and there is much arbitrariness in terms of picking the
values of N (ℓ) and the so-called activation functions h(ℓ)
i (ℓ) . A possible choice
(out of myriad others) is as follows (see, e.g., Nakano et al. 2018)10:
9We suppress the time variable t for the sake of notational simplicity.
10Again, there is no magic bullet here. A priori, a host of activation functions can be used, e.g., sigmoid
(a.k.a. logistic), tanh (hyperbolic tangent), rectiﬁed linear unit (ReLU), softmax, etc. For some pertinent
literature, see, e.g., Bengio ( 2009), Chandra ( 2003), da S. Gomes et al. ( 2011), Glorot et al. ( 2011),

18 Cryptocurrencies 253
h(ℓ)
i (ℓ) ( ⃗Y (ℓ) ) = max
(
Y (ℓ)
i (ℓ) , 0
)
,ℓ = 2,..., L − 1 (ReLU) (18.14)
h(L)
i (L)( ⃗Y (L)) = Y (L)
i (L)
⎡
⎣
N (L)
∑
j (L)=1
Y (L)
j (L)
⎤
⎦
−1
(softmax) (18.15)
I.e., ReLU is used at the hidden layers (and the algorithm moves onto the next
layer only if some neurons are activated (ﬁred) at layer ℓ, i.e., at least some
Y (ℓ)
i (ℓ) > 0), and softmax is used at the output layer (so that we have the condition
(18.11) by construction). Further, to train the model, i.e., to determine the
unknown parameters, some kind of error function E (we suppress its variables)
must be minimized, e.g., the so-called cross-entropy (see, e.g., de Boer et al.
2005):
E =−
∑
t ∈Dtrain
K∑
α=1
Sα(t ) ln(pα(t )) ( 18.16)
T o minimize E , one can, e.g., use the stochastic gradient descent (SGD)
method, which minimizes the error function iteratively until the procedure
converges.11
Finally, we must specify the trading rules. There are a number of possibilities
here depending on the number of quantiles, i.e., the choice of K . A reasonable
trading signal is given by:
Signal =
{
Buy, iff max (pα(t )) = pK (t )
Sell, iff max (pα(t )) = p1(t ) (18.17)
Therefore, the trader buys BTC if the predicted class is pK (t )(the top quantile),
and sells if it is p1(t ) (the bottom quantile). This trading rule can be modiﬁed.
E.g., the buy signal can be based on the top 2 quantiles, and the sell signal can
be based on the bottom 2 quantiles (see, e.g., Nakano et al. 2018).12
Goodfellow et al. ( 2013), Karlik and V ehbi ( 2011), Mhaskar and Micchelli ( 1993), Singh and Chandra
(2003), Wu ( 2009).
11A variety of methods can be used. For some pertinent literature, see, e.g., Denton and Hung ( 1996),
Dong and Zhou ( 2008), Dreyfus (1990), Ghosh (2012), Kingma and Ba ( 2014), Ruder (2017), Rumelhart
et al. ( 1986), Schmidhuber ( 2015), Wilson et al. ( 2018).
12Various techniques used in applying ANNs to other asset classes such as equities may also be useful for
cryptocurrencies. See, e.g., Ballings et al. ( 2015), Chong et al. ( 2017), Dash and Dash ( 2016), de Oliveira
et al. ( 2013), Sezer et al. ( 2017), Yao et al. ( 1999). For some additional literature, see fn. 27 in Chapter 3.

254 Z. Kakushadze and J. A. Serur
18.3 Strategy: Sentiment Analysis—Naïve
Bayes Bernoulli
Social media sentiment analysis based strategies have been used in stock trad-
ing13 and also applied to cryptocurrency trading. The premise is to use a
machine learning classiﬁcation scheme to forecast, e.g., the direction of the
BTC price movement based on tweet data. This entails collecting all tweets
containing at least one keyword from a pertinent (to BTC price forecast-
ing) learning vocabulary V over some timeframe, and cleaning this data.
14
The resultant data is then further processed by assigning a so-called feature
(M-vector) Xi to each tweet labeled by i = 1,..., N ,w h e r eN is the number
of tweets in the dataset. Here M =| V | is the number of keywords in the
learning vocabulary V . So, the components of each vector Xi are Xia ,w h e r e
a = 1,..., M labels the words in V . Thus, if the word wa ∈ V labeled by
a is not present in the tweet Ti labeled by i ,t h e n Xia = 0.I f wa is present
in Ti , then we can set Xia = 1 or Xia = nia ,w h e r e nia counts the number
of times wa appears in Ti . In the former case (which is what we focus on in
the following) we have a Bernoulli probability distribution, while in the latter
case we have a multinomial distribution.
Next, we need to build a model that, given the N feature vectors Xi , predicts
one out of a preset number K of outcomes (so-called classes) Cα, α = 1,..., K .
E.g., we can have K = 2 outcomes, whereby BTC is forecasted to go up or
down, which can be used as the buy/sell signal. Alternatively, as in the ANN
strategy in Sect. 18.2, we can have K quantiles for the normalized returns
ˆR(t ), etc. This then deﬁnes our trading rules. Once the classes Cα are chosen,
a simple way to forecast them is to build a model for conditional probabil-
ities P(Cα|X1,..., X N ). Here, generally, P(A|B) denotes the conditional
probability of A occurring assuming B is true. Pursuant to Bayes’ theorem, we
have
P(A|B) = P(B|A) P(A)
P(B) (18.18)
13For some literature, see, e.g., Bollen and Mao ( 2011), Bollen et al. ( 2011), Kordonis et al. ( 2016), Liew
and Budavári ( 2016), Mittal and Goel ( 2012), Nisar and Yeung ( 2018), Pagolu et al. ( 2016), Rao and
Srivastava ( 2012), Ruan et al. ( 2018), Sprenger et al. ( 2014), Sul et al. ( 2017), Zhang et al. ( 2011).
14This, among other things, includes removing duplicate tweets likely generated by ubiquitous T witter
bots, removing the so-called stop-words (e.g., “the”, “is”, “in”,“which”, etc.), which do not add value, from
the tweets, and performing the so-called stemming, i.e., reducing words to their base form (e.g., “investing”
and “invested” are reduced to “invest”, etc.). The latter can be achieved using, e.g., the Porter stemming
algorithm or other similar algorithms (for some literature, see, e.g., Hull [ 1996], Porter [ 1980], Raulji and
Saini [ 2016], Willett [ 2006]).

18 Cryptocurrencies 255
where P(A) and P(B) are the probabilities of A and B occurring indepen-
dently of each other. So, we have
P(Cα|X1,..., X N ) = P(X1,..., X N |Cα) P(Cα)
P(X1,..., X N ) (18.19)
Note that P(X1,..., X N ) is independent of Cα and will not be important
below. Now, P(Cα) can be estimated from the training data. The primary dif-
ﬁculty is in estimating P(X1,..., X N |Cα). Here a simpliﬁcation occurs if we
make the “naïve” conditional independence assumption (hence the term“naïve
Bayes”), i.e., that, given the class Cα,f o ra l l i the feature Xi is conditionally
independent of every other feature X j , j = 1,..., N ( j ̸= i ):
P(Xi |Cα, X1,..., Xi −1, Xi +1,..., X N ) = P(Xi |Cα)( 18.20)
Then Eq. ( 18.19) simpliﬁes as follows:
P(Cα|X1,..., X N ) = γ P(Cα)
N∏
i =1
P(Xi |Cα)( 18.21)
γ = 1/P(X1,..., X N )( 18.22)
The conditional probabilities P(Xi |Cα)can be estimated using the conditional
probabilities P(wa |Cα) for the M words wa in the learning vocabulary V :
P(Xi |Cα) =
M∏
a=1
Qia α (18.23)
Qia α = P(wa |Cα), Xia = 1 (18.24)
Qia α = 1 − P(wa |Cα), Xia = 0 (18.25)
The conditional probabilities P(wa |Cα) can simply be estimated based on
the occurrence frequencies of the words wa in the training data. Similarly,
the probabilities P(Cα) can be estimated from the training data. 15 So, if we
15For some literature on applying T witter sentiment to Bitcoin trading, see, e.g., Colianni et al. ( 2015),
Georgoula et al. ( 2015), which also discuss other machine learning methods such as support vector
machines (SVM) and logistic regression (a.k.a. logit model). For some literature on Bitcoin trading using
other sentiment data, see, e.g., Garcia and Schweitzer ( 2015), Li et al. ( 2018). For some literature on
applying tree boosting algorithms to cryptocurrency trading, see, e.g., Alessandretti et al. ( 2018), Li et al.
(2018). For some additional pertinent literature (which generally appears to be relatively scarce for BTC
compared with similar literature on stock trading), see, e.g., Amjad and Shah ( 2017), Jiang and Liang
(2017), Shah and Zhang ( 2014).

256 Z. Kakushadze and J. A. Serur
set the forecasted value C pr ed of the outcome to that with the maximum
P(Cα|X1,..., X N ),t h e n
C pr ed = argmax Cα∈{1,...,K } P(Cα)
N∏
i =1
M∏
a=1
[P(wa |Cα)]Xia [1 − P(wa |Cα)]1−Xia
(18.26)
References
Alessandretti, L., ElBahrawy, A., Aiello, L. M., & Baronchelli, A. (2018). Machine
Learning the Cryptocurrency Market (Working Paper). Available online: https://
arxiv.org/pdf/1805.08550.pdf.
Amjad, M. J., & Shah, D. (2017). T rading Bitcoin and Online Time Series Prediction
(Working Paper). Available online: http://proceedings.mlr.press/v55/amjad16.pdf.
Baek, C., & Elbeck, M. (2014). Bitcoins as an Investment or Speculative V ehicle? A
First Look. Applied Economics Letters , 22(1), 30–34.
Ballings, M., Van den Poel, D., Hespeels, N., & Gryp, R. (2015). Evaluating Multiple
Classiﬁers for Stock Price Direction Prediction. Expert Systems with Applications ,
42(20), 7046–7056.
Bariviera, A. F ., Basgall, M. J., Hasperué, W ., & Naiouf, M. (2017). Some Stylized
Facts of the Bitcoin Market. Physica A: Statistical Mechanics and Its Applications ,
484, 82–90.
Bengio, Y. (2009). Learning Deep Architectures for AI. Foundations and T rends in
Machine Learning , 2(1), 1–127.
Bollen, J., & Mao, H. (2011). T witter Mood as a Stock Market Predictor. Computer,
44 (10), 91–94.
Bollen, J., Mao, H., & Zeng, X. (2011). T witter Mood Predicts the Stock Market.
Journal of Computational Science , 2(1), 1–8.
Bouoiyour, J., Selmi, R., & Tiwari, A. K. (2015). Is Bitcoin Business Income or
Speculative Foolery? New Ideas Through an Improved Frequency Domain Analysis.
Annals of Financial Economics , 10 (1), 1–23.
Bouoiyour, J., Selmi, R., Tiwari, A. K., & Olayeni, O. R. (2016). What Drives Bitcoin
Price? Economics Bulletin , 36 (2), 843–850.
Bouri, E., Gupta, R., Tiwari, A. K., & Roubaud, D. (2017a). Does Bitcoin Hedge
Global Uncertainty? Evidence from Wavelet-Based Quantile-in-Quantile Regres-
sions. Finance Research Letters , 23, 87–95.
Bouri, E., Molnár, P ., Azzi, G., Roubaud, D., & Hagfors, L. I. (2017b). On the Hedge
and Safe Haven Properties of Bitcoin: Is It Really More Than a Diversiﬁer? Finance
Research Letters , 20, 192–198.

18 Cryptocurrencies 257
Brandvold, M., Molnár, P ., Vagstad, K., & Valstad, O. C. A. (2015). Price Discovery
on Bitcoin Exchanges. Journal of International Financial Markets, Institutions and
Money, 36, 18–35.
Brière, M., Oosterlinck, K., & Szafarz, A. (2015). Virtual Currency, T angible Return:
Portfolio Diversiﬁcation with Bitcoin. Journal of Asset Management , 16 (6), 365–
373.
Chandra, P . (2003). Sigmoidal Function Classes for Feedforward Artiﬁcial Neural
Networks. Neural Processing Letters , 18(3), 205–215.
Cheah, E. T ., & Fry, J. (2015). Speculative Bubbles in Bitcoin Markets? An Empirical
Investigation into the Fundamental Value of Bitcoin. Economics Letters, 130, 32–36.
Cheung, A., Roca, E., & Su, J.-J. (2015). Crypto-currency Bubbles: An Application
of the Phillips-Shi-Yu (2013) Methodology on Mt. Gox Bitcoin Prices. Applied
Economics, 47 (23), 2348–2358.
Chong, E., Han, C., & Park, F . C. (2017). Deep Learning Networks for Stock Market
Analysis and Prediction: Methodology, Data Representations, and Case Studies.
Expert Systems with Applications , 83, 187–205.
Ciaian, P ., Rajcaniova, M., & Kancs, D. (2015). The Economics of BitCoin Price
Formation. Applied Economics , 48(19), 1799–1815.
Colianni, S., Rosales, S., & Signorotti, M. (2015). Algorithmic T rading of Cryptocur-
rency Based on T witter Sentiment Analysis (Working Paper). Available online: http://
cs229.stanford.edu/proj2015/029_report.pdf .
da S. Gomes, G. S., Ludermir, T . B., & Lima, L. M. M. R. (2011). Comparison
of New Activation Functions in Neural Network for Forecasting Financial Time
Series. Neural Computing and Applications , 20 (3), 417–439.
Dash, R., & Dash, P . K. (2016). A Hybrid Stock T rading Framework Integrating
T echnical Analysis with Machine Learning T echniques.Journal of Finance and Data
Science, 2(1), 42–57.
de Boer, P .-T ., Kroese, D. P ., Mannor, S., & Rubinstein, R. Y. (2005). A T utorial on
the Cross-Entropy Method. Annals of Operations Research , 134 (1), 19–67.
de Oliveira, F . A., Nobre, C. N., & Zárate, L. E. (2013). Applying Artiﬁcial Neu-
ral Networks to Prediction of Stock Price and Improvement of the Directional
Prediction Index—Case Study of PETR4, Petrobras, Brazil. Expert Systems with
Applications, 40 (18), 7596–7606.
Denton, J. W ., & Hung, M. S. (1996). A Comparison of Nonlinear Optimization
Methods for Supervised Learning in Multilayer Feedforward Neural Networks.
European Journal of Operational Research , 93(2), 358–368.
Dong, Z., & Zhou, D.-X. (2008). Learning Gradients by a Gradient Descent Algo-
rithm. Journal of Mathematical Analysis and Applications , 341(2), 1018–1027.
Donier, J., & Bouchaud, J.-P . (2015). Why Do Markets Crash? Bitcoin Data Offers
Unprecedented Insights. PLoS ONE , 10 (10), e0139356.
Dowd, K., & Hutchinson, M. (2015). Bitcoin Will Bite the Dust. Cato Journal, 35 (2),
357–382.

258 Z. Kakushadze and J. A. Serur
Dreyfus, S. E. (1990). Artiﬁcial Neural Networks, Back Propagation, and the Kelley-
Bryson Gradient Procedure. Journal of Guidance, Control, and Dynamics , 13(5),
926–928.
Dyhrberg, A. H. (2015). Bitcoin, Gold and the Dollar—A GARCH V olatility Anal-
ysis. Finance Research Letters , 16, 85–92.
Dyhrberg, A. H. (2016). Hedging Capabilities of Bitcoin. Is It the Virtual Gold?
Finance Research Letters , 16, 139–144.
Eisl, A., Gasser, S., & Weinmayer, K. (2015). Caveat Emptor: Does Bitcoin Improve
Portfolio Diversiﬁcation? (Working Paper). Available online: https://ssrn.com/
abstract=2408997.
Fry, J., & Cheah, E. T . (2016). Negative Bubbles and Shocks in Cryptocurrency
Markets. International Review of Financial Analysis , 47, 343–352.
Gajardo, G., Kristjanpoller, W . D., & Minutolo, M. (2018). Does Bitcoin Exhibit
the Same Asymmetric Multifractal Cross-correlations with Crude Oil, Gold and
DJIA as the Euro, Great British Pound and Yen? Chaos, Solitons & Fractals , 109,
195–205.
Garcia, D., & Schweitzer, F . (2015). Social Signals and Algorithmic T rading of Bitcoin.
Royal Society Open Science , 2(9), 150288.
Garcia, D., T essone, C. J., Mavrodiev, P ., & Perony, N. (2014). The Digital T races of
Bubbles: Feedback Cycles Between Socioeconomic Signals in the Bitcoin Economy.
Journal of The Royal Society Interface , 11(99), 0623.
Georgoula, I., Pournarakis, D., Bilanakos, C., Sotiropoulos, D., & Giaglis, G. M.
(2015). Using Time-Series and Sentiment Analysis to Detect the Determinants of Bit-
coin Prices (Working Paper). Available online: https://ssrn.com/abstract=2607167.
Ghosh, A. (2012). Comparative Study of Financial Time Series Prediction by Artiﬁcial
Neural Network with Gradient Descent Learning. International Journal of Scientiﬁc
& Engineering Research , 3(1), 41–49.
Glorot, X., Bordes, A., & Bengio, Y. (2011). Deep Sparse Rectiﬁer Neural Networks.
Proceedings of Machine Learning Research , 15, 315–323.
Goodfellow, I., Warde-Farley, D., Mirza, M., Courville, A., & Bengio, Y. (2013).
Maxout Networks. Proceedings of Machine Learning Research , 28(3), 1319–1327.
Harvey, C. R. (2014). Bitcoin Myths and Facts (Working Paper). Available online:
https://ssrn.com/abstract=2479670.
Harvey, C. R. (2016). Cryptoﬁnance (Working Paper). Available online: https://ssrn.
com/abstract=2438299.
Hull, D. A. (1996). Stemming Algorithms: A Case Study for Detailed Evaluation.
Journal of the American Society for Information Science and T echnology , 47 (1), 70–
84.
Jiang, Z., & Liang, J. (2017). Cryptocurrency Portfolio Management with Deep Rein-
forcement Learning (Working Paper). Available online: https://arxiv.org/pdf/1612.
01277.pdf .
Karlik, B., & V ehbi, A. (2011). Performance Analysis of Various Activation Functions
in Generalized MLP Architectures of Neural Networks. International Journal of
Artiﬁcial Intelligence and Expert Systems , 1(4), 111–122.

18 Cryptocurrencies 259
Kim, Y. B., Kim, J. G., Kim, W ., Im, J. H., Kim, T . H., Kang, S. J., et al. (2016).
Predicting Fluctuations in Cryptocurrency T ransactions Based on User Comments
and Replies. PLoS ONE , 11(8), e0161197.
Kingma, D. P ., & Ba, J. (2014). Adam: A Method for Stochastic Optimization (Working
Paper). Available online: https://arxiv.org/pdf/1412.6980.
Kordonis, J., Symeonidis, A., & Arampatzis, A. (2016). Stock Price Forecasting via
Sentiment Analysis on T witter. In Proceedings of the 20th Pan-Hellenic Conference
on Informatics (PCI’16) (Article No. 36). New York, NY: ACM.
Kristoufek, L. (2015). What Are the Main Drivers of the Bitcoin Price? Evidence from
Wavelet Coherence Analysis. PLoS ONE , 10 (4), e0123923.
Lee, D. K. C., Guo, L., & Wang, Y. (2018). Cryptocurrency: A New Investment
Opportunity? Journal of Alternative Investments , 20 (3), 16–40.
Li, T . R., Chamrajnagar, A. S., Fong, X. R., Rizik, N. R., & Fu, F . (2018). Sentiment-
Based Prediction of Alternative Cryptocurrency Price Fluctuations Using Gradient
Boosting T ree Model (Working Paper). Available online: https://arxiv.org/pdf/1805.
00558.pdf .
Liew, J. K.-S., & Budavári, T . (2016). Do T weet Sentiments Still Predict the Stock Market?
(Working Paper). Available online: https://ssrn.com/abstract=2820269.
Liew, J. K.-S., Li, R. Z., & Budavári, T . (2018). Crypto-currency Investing Examined
(Working Paper). Available online: https://ssrn.com/abstract=3157926.
Mhaskar, H. N., & Micchelli, C. A. (1993). How to Choose an Activation Func-
tion. In Proceedings of the 6th International Conference on Neural Information Pro-
cessing Systems (NIPS’93) (pp. 319–326). San Francisco, CA: Morgan Kaufmann
Publishers.
Mittal, A., & Goel, A. (2012). Stock Prediction Using T witter Sentiment Analysis (Work-
ing Paper). Palo Alto, CA: Stanford University.
Nakamoto, S. (2008). Bitcoin: A Peer-to-Peer Electronic Cash System (Working Paper).
Available online: https://bitcoin.org/bitcoin.pdf .
Nakano, M., T akahashi, A., & T akahashi, S. (2018). Bitcoin T echnical T rading with
Artiﬁcial Neural Network (Working Paper). Available online: https://ssrn.com/
abstract=3128726.
Nisar, T . M., & Yeung, M. (2018). T witter as a T ool for Forecasting Stock Market
Movements: A Short-Window Event Study. Journal of Finance and Data Science ,
4 (2), 101–119.
Ortisi, M. (2016). Bitcoin Market V olatility Analysis Using Grand Canonical Minority
Game. Ledger, 1, 111–118.
Pagolu, V . S., Reddy, K. N., Panda, G., & Majhi, B. (2016). Sentiment Analysis of
T witter Data for Predicting Stock Market Movements. In Proceedings of the 2016
International Conference on Signal Processing, Communication, Power and Embedded
System (SCOPES) (pp. 1345–1350). Washington, DC: IEEE.
Porter, M. F . (1980). An Algorithm for Sufﬁx Stripping. Program, 14 (3), 130–137.
Rao, T ., & Srivastava, S. (2012). Analyzing Stock Market Movements Using T witter
Sentiment Analysis. In Proceedings of the 2012 International Conference on Advances

260 Z. Kakushadze and J. A. Serur
in Social Networks Analysis and Mining (ASONAM 2012) (pp. 119–123). Wash-
ington, DC: IEEE.
Raulji, J. K., & Saini, J. R. (2016). Stop-Word Removal Algorithm and Its Imple-
mentation for Sanskrit Language. International Journal of Computer Applications ,
150 (2), 15–17.
Ruan, Y., Durresi, A., & Alfantoukh, L. (2018). Using T witter T rust Network for
Stock Market Analysis. Knowledge-Based Systems , 145, 207–218.
Ruder, S. (2017). An Overview of Gradient Descent Optimization Algorithms (Working
Paper). Available online: https://arxiv.org/pdf/1609.04747.pdf.
Rumelhart, D. E., Hinton, G. E., & Williams, R. J. (1986). Learning Representations
by Back-Propagating Errors. Nature, 323(6088), 533–536.
Schmidhuber, J. (2015). Deep Learning in Neural Networks: An Overview. Neural
Networks, 61, 85–117.
Sezer, O. B., Ozbayoglu, M., & Dogdu, E. (2017). A Deep Neural-Network Based
Stock T rading System Based on Evolutionary Optimized T echnical Analysis Param-
eters. Procedia Computer Science , 114, 473–480.
Shah, D., & Zhang, K. (2014). Bayesian Regression and Bitcoin (Working Paper).
Available online: https://arxiv.org/pdf/1410.1231.pdf.
Singh, Y., & Chandra, P . (2003). A Class +1 Sigmoidal Activation Functions for
FFANNs. Journal of Economic Dynamics and Control , 28(1), 183–187.
Sprenger, T . O., T umasjan, A., Sandner, P . G., & Welpe, I. M. (2014). T weets and
T rades: The Information Content of Stock Microblogs. European Financial Man-
agement, 20 (5), 926–957.
Sul, H. K., Dennis, A. R., & Yuan, L. (I). (2017). T rading on T witter: Using Social
Media Sentiment to Predict Stock Returns. Decision Sciences , 48(3), 454–488.
Van Alstyne, M. (2014). Why Bitcoin Has Value. Communications of the ACM , 57 (5),
30–32.
Wang, S., & V ergne, J.-P . (2017). Buzz Factor or Innovation Potential: What Explains
Cryptocurrencies Returns? PLoS ONE , 12(1), e0169556.
White, L. H. (2015). The Market for Cryptocurrencies. Cato Journal, 35 (2), 383–402.
Wilder, J. W ., Jr. (1978). New Concepts in T echnical T rading Systems. Greensboro, NC:
T rend Research.
Willett, P . (2006). The Porter Stemming Algorithm: Then and Now. Program: Elec-
tronic Library and Information Systems , 40(3), 219–223.
Wilson, A. C., Roelofs, R., Stern, M., Stern, N., & Recht, B. (2018). The Marginal
Value of Adaptive Gradient Methods in Machine Learning (Working Paper). Available
online: https://arxiv.org/pdf/1705.08292.pdf.
Wu, H. (2009). Global Stability Analysis of a General Class of Discontinuous Neural
Networks with Linear Growth Activation Functions. Information Sciences, 179
(19),
3432–3441.
Yao, J., T an, C. L., & Poh, H. L. (1999). Neural Networks for T echnical Analysis:
A Study on KLCI. International Journal of Theoretical and Applied Finance , 2(2),
221–241.

18 Cryptocurrencies 261
Zhang, X., Fuehres, H., & Gloor, P . A. (2011). Predicting Stock Market Indicators
Through T witter “I hope it is not as bad as I fear”. Procedia—Social and Behavioral
Sciences, 26, 55–62.
module ResidualActivationChainScenario

import "./residual_activation_chain.res"

experiment empty {
    seeds { }
    query residual completed(item) @ after tau
}
experiment real {
    seeds { starts(item) @ tau }
    query residual completed(item) @ after tau
}
experiment negative {
    seeds { ~starts(item) @ tau }
    query residual completed(item) @ after tau
}
experiment wrongTime {
    seeds { starts(item) @ sigma }
    query residual completed(item) @ after tau
}
experiment wrongObject {
    seeds { starts(other) @ tau }
    query residual completed(item) @ after tau
}

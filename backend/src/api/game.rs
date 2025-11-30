// main api, everyone gets this
trait GlobalApi {}

// miner api
trait MineConductor {
    fn tick();
}

// logistics api
trait LogisticsConductor {}

// defender api
trait DefenderConductor {}

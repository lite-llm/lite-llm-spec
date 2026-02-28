//! Type definitions for Lite LLM formal model.
//!
//! This module defines the core types used in the formal
//! model of the Lite LLM architecture. The types mirror
//! the mathematical structures described in the design
//! specification, such as tiers, groups, experts, routing
//! selections and deterministic seed handling. These
//! definitions are intended to serve as a reference
//! implementation for the architecture and can be adapted
//! into a larger codebase.

// NOTE: This file is self‑contained for illustrative purposes.
// In a real codebase you would import Tensor and Result from
// the core crate or define them in a shared module. Here we
// provide minimal stubs so that the code compiles without
// depending on external modules.

/// Placeholder error type used by traits in this module.
///
/// In a full implementation this would be replaced by
/// `crate::core::error::Error` and `Result<T>` would alias
/// `std::result::Result<T, crate::core::error::Error>`.
#[derive(Debug)]
pub enum Error {
    /// Generic error with a message.
    Msg(String),
    /// Feature is unimplemented.
    Unimplemented(String),
}

/// Convenience result type alias used by this module.
pub type Result<T> = std::result::Result<T, Error>;

/// A very minimal representation of a tensor.
///
/// For the purposes of type mapping, this struct only
/// includes a shape. In a real system it would manage
/// memory across devices, data type information, etc.
#[derive(Clone, Debug)]
pub struct Tensor {
    /// The shape of the tensor as a list of dimensions.
    pub shape: Vec<usize>,
}

impl Tensor {
    /// Construct a tensor with the given shape.
    pub fn new(shape: &[usize]) -> Self {
        Tensor { shape: shape.to_vec() }
    }

    /// Returns the batch size, assumed to be the first dimension.
    pub fn batch_size(&self) -> Option<usize> {
        self.shape.get(0).copied()
    }
}

/// A unique identifier for a parameter tier.
///
/// Tiers represent disjoint capacity slices of the model. Each
/// tier may hold a different number of parameters (e.g., 1B,
/// 10B, 100B). When a new tier is added to the system, a new
/// `TierId` should be created with a distinct integer value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TierId(pub u16);

impl TierId {
    /// Creates a new `TierId` from a numeric identifier.
    pub const fn new(id: u16) -> Self {
        TierId(id)
    }
}

/// Possible storage placement hints for a tier.
///
/// Placement hints inform the runtime where the weights of a
/// given tier are expected to reside. These are only hints and
/// the actual runtime may move data between tiers as needed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlacementHint {
    /// The tier is kept in fast on‑device memory (HBM/GPU memory).
    Hot,
    /// The tier resides in main memory (DDR/DRAM).
    Warm,
    /// The tier resides on secondary storage (NVMe SSD).
    Cold,
    /// The tier is archived on remote object storage (e.g. S3).
    Archive,
}

/// Configuration for a single parameter tier.
///
/// Each tier may have a different number of expert groups and
/// experts per group. The `placement` field suggests where the
/// weights for the tier should be stored by default.
#[derive(Clone, Debug)]
pub struct TierConfig {
    /// Unique identifier for the tier.
    pub id: TierId,
    /// Number of groups within this tier (G_t in the formal model).
    pub groups: usize,
    /// Number of experts in each group (E_{t,g} in the formal model).
    pub experts_per_group: usize,
    /// Hint to the runtime about where to place the experts for this tier.
    pub placement: PlacementHint,
    /// Optional metadata about the tier (for example, domain specialisation).
    pub metadata: Option<String>,
}

/// A selection of tiers available for a given inference or training run.
///
/// The `tiers` vector lists exactly which tiers may activate. If
/// `cumulative` is true, any tier with an ID lower than or equal to
/// those listed is considered implicitly included.
#[derive(Clone, Debug)]
pub struct TierSet {
    /// The list of tiers allowed by this set.
    pub tiers: Vec<TierId>,
    /// Whether to treat the list cumulatively (include all smaller tiers).
    pub cumulative: bool,
}

impl TierSet {
    /// Check whether a given tier is allowed under this set.
    pub fn contains(&self, tier: TierId) -> bool {
        if self.cumulative {
            // cumulative means include all tiers with id <= the max id in `tiers`
            let max = self.tiers.iter().max_by_key(|t| t.0);
            match max {
                Some(max_id) => tier.0 <= max_id.0,
                None => false,
            }
        } else {
            self.tiers.contains(&tier)
        }
    }
}

/// Routing cardinality hyperparameters.
///
/// These values control how many selections are made at each level
/// of hierarchical sparse expert routing. The product
/// `k_tier * k_group * k_expert` bounds the maximum number of experts
/// activated per token.
#[derive(Clone, Copy, Debug)]
pub struct RoutingConfig {
    /// Maximum number of tiers selected at level 1.
    pub k_tier: usize,
    /// Maximum number of groups selected within each tier at level 2.
    pub k_group: usize,
    /// Maximum number of experts selected within each group at level 3.
    pub k_expert: usize,
}

impl RoutingConfig {
    /// Compute the maximum number of active experts per token.
    pub fn max_active_experts(&self) -> usize {
        self.k_tier * self.k_group * self.k_expert
    }
}

/// Unique coordinates of an expert in the three‑level hierarchy.
///
/// Each expert is identified by its tier ID, its group index within the
/// tier, and its expert index within the group. The indices are
/// represented as `u32` to allow for large numbers of groups/experts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ExpertKey {
    /// Identifier of the tier to which this expert belongs.
    pub tier: TierId,
    /// Zero‑based index of the group within the tier.
    pub group: u32,
    /// Zero‑based index of the expert within the group.
    pub expert: u32,
}

/// A routing assignment for a single token.
///
/// Each route indicates which expert should process the token, along
/// with a weight (probability or mixture coefficient) and a
/// priority hint. The priority can be used to order prefetching or
/// caching operations.
#[derive(Clone, Debug)]
pub struct Route {
    /// The key of the selected expert.
    pub expert: ExpertKey,
    /// The mixing weight associated with this assignment.
    pub weight: f32,
    /// A small integer priority used for cache hinting.
    pub priority: u8,
}

/// The result of routing a batch of tokens at one layer.
///
/// For each token in the batch, a vector of `Route` entries is
/// produced. The outer vector length equals the batch size. The inner
/// vectors have length at most `k_tier * k_group * k_expert`.
pub type RoutingResult = Vec<Vec<Route>>;

/// Trait defining the behaviour of a router at one transformer layer.
///
/// A router receives a batch of hidden states (shape [batch, d]) and
/// a `TierSet`. It returns the routing decisions for each token.
///
/// The router is responsible for implementing the three‑level
/// selection: tiers, groups within tiers, and experts within groups.
/// The determinism contract requires that the same inputs always
/// produce the same routing result when given the same seed.
pub trait Router: Send + Sync {
    /// Compute the routing assignments for a batch of tokens.
    ///
    /// * `h` - The input hidden tensor of shape [batch, d].
    /// * `tiers` - The set of tiers allowed in this context.
    ///
    /// Returns a `RoutingResult` with the same number of entries as
    /// the batch dimension of `h`.
    fn route(&self, h: &Tensor, tiers: &TierSet) -> Result<RoutingResult>;
}

/// Trait defining the behaviour of an expert network.
///
/// Experts implement a small feedforward function from a tensor to a
/// tensor. They are stateless and may be sharded or offloaded
/// transparently by the runtime. Errors may be returned to signal
/// loading or execution failures.
pub trait Expert: Send + Sync {
    /// Forward pass of the expert on a batch of inputs.
    ///
    /// * `x` - Tensor of shape [batch, d] representing the inputs for
    ///   this expert.
    ///
    /// Returns a tensor of shape [batch, d] containing the outputs.
    fn forward(&self, x: &Tensor) -> Result<Tensor>;
}

/// Interface for retrieving experts from storage.
///
/// The expert store manages all experts across tiers and groups. It
/// hides the details of loading weights from different storage
/// levels and caching them in fast memory. The store must respect
/// placement hints, but is free to move data to satisfy requests.
pub trait ExpertStore: Send + Sync {
    /// Obtain an expert instance by its key.
    ///
    /// The returned expert should be ready to perform a forward
    /// computation. If the expert must be loaded from slow storage,
    /// this method may block or asynchronously load the weights.
    fn get_expert(&self, key: ExpertKey) -> Result<Box<dyn Expert>>;

    /// Query the placement hint for a particular tier.
    ///
    /// This allows the runtime to inspect where tiers are intended
    /// to reside and to prefetch accordingly.
    fn placement_hint(&self, tier: TierId) -> PlacementHint;
}

/// A mixture‑of‑experts (MoE) layer.
///
/// This structure bundles together a router and an expert store and
/// exposes a `forward` method to compute the MoE output. The
/// dispatching and combination logic, including communication
/// primitives, is left to a concrete implementation.
pub struct MoELayer<R: Router, S: ExpertStore> {
    /// The router used to produce expert assignments.
    pub router: R,
    /// The store used to retrieve expert implementations.
    pub store: S,
    /// Routing cardinality hyperparameters.
    pub routing_cfg: RoutingConfig,
}

impl<R: Router, S: ExpertStore> MoELayer<R, S> {
    /// Construct a new MoE layer from its components.
    pub fn new(router: R, store: S, routing_cfg: RoutingConfig) -> Self {
        MoELayer { router, store, routing_cfg }
    }

    /// Execute the MoE layer on the given hidden states.
    ///
    /// This method performs the routing and dispatch phases. It is
    /// responsible for packing tokens by expert, invoking the experts,
    /// and combining the results according to the routing weights.
    /// For brevity, the dispatch/combine mechanics are not
    /// implemented here; instead, an error is returned.
    pub fn forward(&self, h: &Tensor, tiers: &TierSet) -> Result<Tensor> {
        // Compute routing assignments for each token.
        let assignments = self.router.route(h, tiers)?;
        // In a full implementation, you would now:
        // 1) For each expert in the assignments, gather the token
        //    inputs destined for that expert into a contiguous buffer.
        // 2) Call `Expert::forward` on the expert with its batch of
        //    inputs.
        // 3) Scatter the outputs back to the original token positions
        //    and combine them weighted by `Route.weight`.
        // This skeleton leaves that logic unimplemented.
        Err(Error::Unimplemented("MoE forward not implemented".into()))
    }
}

/// Global model configuration reflecting the formal design.
///
/// `ModelConfig` encapsulates the high‑level parameters of the
/// Lite LLM architecture. It maps directly to the variables in
/// the formal model such as the number of layers L, the hidden
/// dimension d, routing cardinalities, and the configured tiers.
#[derive(Clone, Debug)]
pub struct ModelConfig {
    /// Number of transformer layers (L).
    pub layers: usize,
    /// Hidden dimension size (d).
    pub hidden_dim: usize,
    /// Routing cardinality hyperparameters.
    pub routing: RoutingConfig,
    /// A list of configured tiers in ascending order of capacity.
    pub tiers: Vec<TierConfig>,
    /// Optional seed for deterministic routing tie‑breaking.
    pub seed: u64,
    /// Additional hyperparameters (attention heads, etc.) can be
    /// stored in future extensions.
}

impl ModelConfig {
    /// Returns the maximum number of active experts per token.
    pub fn max_active_experts(&self) -> usize {
        self.routing.max_active_experts()
    }
}
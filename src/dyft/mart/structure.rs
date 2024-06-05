use crate::config::MartConfig;
use crate::dyft::*;
use crate::point::Distance;

#[derive(GetSize)]
pub struct MartIndex<'a> {
    pub(crate) m_array_2: MartSparse<MartNode2>,
    pub(crate) m_array_4: MartSparse<MartNode4>,
    pub(crate) m_array_8: MartSparse<MartNode8>,
    pub(crate) m_array_16: MartSparse<MartNode16>,
    pub(crate) m_array_32: MartSparse<MartNode32>,
    pub(crate) m_array_64: MartDense<MartNode64>,
    pub(crate) m_array_128: MartDense<MartNode128>,
    pub(crate) m_array_256: MartFull<MartNode256>,
    pub(crate) m_edges: Vec<MartEdge>,
    pub(crate) m_rootptr: MartPointer,
    pub(crate) m_postings_list: SparseTable,
    pub(crate) m_splitthreshold: SplitThresholds<'a>,
    pub(crate) m_split_count: usize,
    pub(crate) m_radius: u32, // radius for hamming distance
    pub(crate) m_errors: u8,  // number of errors in trie search
    pub(crate) m_bit_positions: &'a BitPositionsEntry,
    pub(crate) m_in_weight: f32,
    pub(crate) m_max_depth: usize,
    pub(crate) m_bits: usize,
    pub(crate) m_end: usize,
    pub(crate) m_begin: usize,
    pub(crate) m_ids: MartNodeId, // number of ids
}

impl<'a> MartIndex<'a> {
    pub fn new(
        &MartConfig {
            bits,
            errors,
            radius,
            in_weight,
            splitthreshold,
            distance: _,
        }: &MartConfig,
        bit_pos_begin: usize,
        bit_pos_end: usize,
    ) -> Self {
        let m_array_2 = MartSparse::<MartNode2>::default();
        let m_array_4 = MartSparse::<MartNode4>::default();
        let m_array_8 = MartSparse::<MartNode8>::default();
        let m_array_16 = MartSparse::<MartNode16>::default();
        let m_array_32 = MartSparse::<MartNode32>::default();
        let m_array_64 = MartDense::<MartNode64>::default();
        let m_array_128 = MartDense::<MartNode128>::default();
        let mut m_array_256 = MartFull::<MartNode256>::default();
        let m_rootptr = m_array_256.make_node();
        let m_edges = Vec::with_capacity(256);
        let m_postings_list = SparseTable::default();
        let m_splitthreshold = DyftFactors::split_thresholds(splitthreshold, bits, errors);
        let m_bit_positions = HamTables::bit_positions(bits);
        MartIndex::<'a> {
            m_splitthreshold,
            m_array_2,
            m_array_4,
            m_array_8,
            m_array_16,
            m_array_32,
            m_array_64,
            m_array_128,
            m_array_256,
            m_edges,
            m_split_count: 0,
            m_rootptr,
            m_postings_list,
            m_ids: 0,
            m_radius: radius,
            m_errors: errors,
            m_bit_positions,
            m_in_weight: in_weight.unwrap_or(1.0),
            m_max_depth: 0,
            m_bits: bits,
            m_begin: bit_pos_begin,
            m_end: bit_pos_end,
        }
    }

    pub fn trie_query<T>(
        &'a self,
        vcodes: &'a VCodeArray<T>,
        qvcodes: &'a VCodeArray<T>,
    ) -> impl Iterator<Item = (usize, usize)> + 'a
    where
        T: VCodeTools,
    {
        qvcodes
            .iter()
            .enumerate()
            .flat_map(|(i, q)| {
                self.trie_search(q)
                    .map(move |candidate| (i, candidate as usize))
            })
            .filter(move |&(query, candidate)| {
                vcodes.verify_candidate_predicate(candidate, qvcodes.access(query), self.m_radius)
            })
    }

    pub fn trie_query_partial_verification<T, V>(
        &'a self,
        vcodes: &'a VCodeArray<T>,
        qvcodes: &'a VCodeArray<T>,
        dataset: &'a [Trajectory],
        queryset: &'a [Trajectory],
        distance: Distance,
    ) -> DyFTPartialVerificationResult<V>
    where
        T: VCodeTools,
        V: FromIterator<(usize, usize)>,
    {
        let partial_verification_iter = qvcodes.iter().enumerate().flat_map(|(i, q)| {
            self.trie_search(q)
                .map(move |candidate| (i, candidate as usize))
        });

        DyFTPartialVerificationResult::<V>::from_candidates(
            partial_verification_iter,
            vcodes,
            qvcodes,
            dataset,
            queryset,
            self.m_radius - (self.m_radius as f32).sqrt().floor() as u32,
            self.m_radius,
            distance,
        )
    }
}

pub struct DyFTPartialVerificationResult<I> {
    m_full_verification_count: usize,
    m_partial_verification_count: usize,
    m_filtered_verification_count: usize,
    m_candidates: I,
}

impl<V> DyFTPartialVerificationResult<V>
where
    V: FromIterator<(usize, usize)>,
{
    pub fn from_candidates<T>(
        candidates: impl Iterator<Item = (usize, usize)>,
        vcodes: &VCodeArray<T>,
        qvcodes: &VCodeArray<T>,
        dataset: &[Trajectory],
        queryset: &[Trajectory],
        hamming_distance: u32,
        upper_hamming_distance: u32,
        distance: Distance,
    ) -> Self
    where
        T: VCodeTools,
    {
        let mut m_full_verification_count = 0;
        let mut m_partial_verification_count = 0;
        let mut m_filtered_verification_count = 0;
        let partial_verification_filter = DyFTPartialVerificationResult::<V>::filter(
            dataset,
            queryset,
            vcodes,
            qvcodes,
            distance,
            hamming_distance,
            upper_hamming_distance,
            &mut m_full_verification_count,
            &mut m_partial_verification_count,
            &mut m_filtered_verification_count,
        );
        let m_candidates = candidates
            .into_iter()
            .filter(partial_verification_filter)
            .collect::<V>();

        DyFTPartialVerificationResult {
            m_full_verification_count,
            m_partial_verification_count,
            m_filtered_verification_count,
            m_candidates,
        }
    }

    pub fn full_verification_count(&self) -> usize {
        self.m_full_verification_count
    }

    pub fn partial_verification_count(&self) -> usize {
        self.m_partial_verification_count
    }

    pub fn filtered_verification_count(&self) -> usize {
        self.m_filtered_verification_count
    }

    pub fn results(self) -> V {
        self.m_candidates
    }

    pub fn filter<'a, T>(
        dataset: &'a [Trajectory],
        queryset: &'a [Trajectory],
        vcodes: &'a VCodeArray<T>,
        qvcodes: &'a VCodeArray<T>,
        distance: Distance,
        hamming_distance: u32,
        upper_hamming_distance: u32,
        full_verification_count: &'a mut usize,
        partial_verification_count: &'a mut usize,
        filtered_verification_count: &'a mut usize,
    ) -> impl for<'b> FnMut(&'b (usize, usize)) -> bool + 'a
    where
        T: VCodeTools,
    {
        move |&(query, candidate)| match vcodes.hamdist_radius(
            candidate,
            qvcodes.access(query),
            hamming_distance,
        ) {
            ham if ham <= hamming_distance => {
                *partial_verification_count += 1;
                true
            }
            ham if ham <= upper_hamming_distance => {
                *full_verification_count += 1;
                queryset[query].frechet_decider(&dataset[candidate], distance)
            }
            _ => {
                *filtered_verification_count += 1;
                false
            }
        }
    }
}

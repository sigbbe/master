use std::fs::File;
use std::sync::Arc;
use anyhow::Error;
use anyhow::Result;
use arrow::array::ArrayRef;
use arrow::array::Float64Array;
use arrow::array::Int64Array;
use geo::algorithm::frechet_distance::FrechetDistance;
use geo::Coord;
use geo::LineString;
use master::config::map_master_path;
use master::config::master_result_dir;
use master::io;
use master::io::write_record_batch;
use master::point::Distance;
use master::trajectory::Trajectory;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;

fn map_to_linestring(t: &Trajectory) -> LineString<Distance> {
    LineString::from_iter(t.iter().map(|p| Coord { x: p.x, y: p.y }))
}

fn create_benchmark(
    dataset: &[LineString<Distance>],
    queryset: &[LineString<Distance>],
) -> Vec<Vec<f64>> {
    let mut matrix = vec![vec![0.0; dataset.len()]; queryset.len()];
    let (sender, receiver) = std::sync::mpsc::channel();
    (0..queryset.len()).into_par_iter().for_each_with(sender, |s, i| {
        let query = &queryset[i];
        dataset.iter().enumerate().for_each(|(j, line)| {
            s.send((i, j, query.frechet_distance(line))).unwrap();
        });
    });
    receiver.into_iter().for_each(|(i, j, v)| matrix[i][j] = v);
    matrix
}

fn save_matrix<P>(
    path: P,
    data_ids: impl Iterator<Item = i64>,
    query_ids: impl Iterator<Item = i64>,
    matrix: Vec<Vec<f64>>,
) -> Result<()>
where
    P: AsRef<std::path::Path>,
{
    let file = File::create(map_master_path(path, master_result_dir()))?;
    let mut batch = vec![
        (
            "id".to_owned(),
            Arc::new(Int64Array::from(data_ids.collect::<Vec<_>>())) as ArrayRef,
        ),
    ];
    batch.extend(
        matrix.into_iter().zip(query_ids)
            .map(|(row, qid)| {
                (
                    qid.to_string(),
                    Arc::new(Float64Array::from(row)) as ArrayRef,
                )
            },
        )
    );
    write_record_batch(batch, file)?;
    Ok(())
}

fn usage() -> Error {
    anyhow::anyhow!("Usage: <dataset> <queryset> <output>")
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let datapath = args.get(1).ok_or_else(usage)?;
    let querypath = args.get(2).ok_or_else(usage)?;
    let outputpath = args.get(3).ok_or_else(usage)?;
    let dataset = io::trajectory_dataset(&datapath)?;
    let data_ids = dataset.ids().into_iter().map(|v| v.value());
    let dataset: Vec<_> = dataset
        .trajectories()
        .into_iter()
        .map(map_to_linestring)
        .collect();
    let queryset = io::trajectory_queryset(&querypath)?;
    let query_ids = queryset.ids().into_iter().map(|v| v.value());
    let queryset: Vec<_> = queryset
        .trajectories()
        .into_iter()
        .map(map_to_linestring)
        .collect();
    save_matrix(
        &outputpath,
        data_ids,
        query_ids,
        create_benchmark(&dataset, &queryset),
    )?;
    Ok(())
}

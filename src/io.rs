#![allow(private_interfaces)]

use super::id::TrajectoryID;
use crate::config::map_master_path;
use crate::config::master_data_dir;
use crate::config::master_query_dir;
use crate::config::master_result_dir;
use crate::dyft::VCodeArray;
use crate::dyft::VCodeTools;
use crate::point::PointMatrix;
use crate::trajectory::Trajectory;
use crate::trajectory::TrajectoryDataset;
use anyhow::anyhow;
use anyhow::Result;
use arrow::array::ArrayRef;
use arrow::array::Float64Array;
use arrow::array::Int64Array;
use arrow::array::PrimitiveArray;
use arrow::array::RecordBatch;
use arrow::array::UInt64Array;
use arrow::buffer::ScalarBuffer;
use arrow::datatypes::ArrowPrimitiveType;
use indexmap::IndexMap;
use num_traits::ToPrimitive;
use parquet::arrow::arrow_reader::ArrowReaderOptions;
use parquet::arrow::arrow_reader::ParquetRecordBatchReader;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::arrow::ArrowWriter;
use parquet::file::properties::WriterProperties;
use parquet::format::FileMetaData;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;

const ID_COL: &str = "id";
const LAT_COL: &str = "lat";
const LON_COL: &str = "lon";

const RESULT_ID_COL: &str = "query";
const RESULT_CANDIDATE_COL: &str = "candidate";

pub fn trajectory_dataset<'a>(path: impl AsRef<Path>) -> Result<TrajectoryDataset<'a>> {
    trajectoryset_parquet(map_master_path(path, master_data_dir()))
}

pub fn trajectory_queryset<'a>(path: impl AsRef<Path>) -> Result<TrajectoryDataset<'a>> {
    trajectoryset_parquet(map_master_path(path, master_query_dir()))
}

fn parquet_reader(path: impl AsRef<Path>) -> Result<ParquetRecordBatchReader> {
    let file = File::open(path)?;
    let options = ArrowReaderOptions::default()
        .with_page_index(true)
        .with_skip_arrow_metadata(false);
    let builder = ParquetRecordBatchReaderBuilder::try_new_with_options(file, options)?;
    builder.build().map_err(|e| e.into())
}

fn trajectoryset_parquet<'a>(path: impl AsRef<Path>) -> Result<TrajectoryDataset<'a>> {
    let mut data: IndexMap<TrajectoryID<'a>, Trajectory> = IndexMap::new();
    let mut reader = parquet_reader(path)?;
    while let Some(Ok(records)) = reader.next() {
        let id_col = records
            .column_by_name(ID_COL)
            .ok_or(anyhow!("No id column"))?;
        let x_col = records
            .column_by_name(LAT_COL)
            .ok_or(anyhow!("No lat column"))?;
        let y_col = records
            .column_by_name(LON_COL)
            .ok_or(anyhow!("No lon column"))?;
        let id_array = id_col
            .as_any()
            .downcast_ref::<Int64Array>()
            .ok_or(anyhow!("id column is not a u64 array"))?;
        let x_array = x_col
            .as_any()
            .downcast_ref::<Float64Array>()
            .ok_or(anyhow!("lat column is not a f64 array"))?;
        let y_array = y_col
            .as_any()
            .downcast_ref::<Float64Array>()
            .ok_or(anyhow!("lon column is not a f64 array"))?;

        for (&id, point) in id_array.values().into_iter().into_iter().zip(
            x_array
                .values()
                .into_iter()
                .zip(y_array.values().into_iter())
                .map(|(&x, &y)| PointMatrix::from([x, y])),
        ) {
            data.entry(id.into())
                .or_insert_with(Trajectory::new)
                .append_point(point);
        }
    }
    Ok(TrajectoryDataset::new(data))
}

pub fn write_query_results<'a, P, I>(path: P, results: I) -> Result<FileMetaData>
where
    I: IntoIterator<Item = (TrajectoryID<'a>, TrajectoryID<'a>)>,
    P: AsRef<Path>,
{
    let (queries, candidates) = results
        .into_iter()
        .map(|(query, candidate)| (query.value(), candidate.value()))
        .unzip::<_, _, Vec<_>, Vec<_>>();
    let queries = Int64Array::from(queries);
    let candidates = Int64Array::from(candidates);
    let file = File::create(map_master_path(path, master_result_dir()))?;
    write_record_batch(
        vec![
            (RESULT_ID_COL, Arc::new(queries) as ArrayRef),
            (RESULT_CANDIDATE_COL, Arc::new(candidates) as ArrayRef),
        ],
        file,
    )
}

pub fn write_vcodes_parquet<const B: usize, T>(
    path: impl AsRef<Path>,
    vcodes: VCodeArray<T>,
) -> Result<FileMetaData>
where
    T: VCodeTools,
{
    let num_codes: usize = vcodes.size();
    let column_iter = (0..vcodes.bits()).map(|i| {
        (
            i.to_string(),
            Arc::new(UInt64Array::new(
                ScalarBuffer::<u64>::from_iter(
                    (0..num_codes)
                        .map(|j| <T as ToPrimitive>::to_u64(&vcodes.access(j)[i]).unwrap()),
                ),
                None,
            )) as ArrayRef,
        )
    });
    let file = File::create(map_master_path(path, master_result_dir()))?;
    write_record_batch(column_iter, file)
}

pub fn write_hashes_parquet<T>(
    path: impl AsRef<Path>,
    hashes: impl Iterator<Item = impl Iterator<Item = T::Native>>,
    ids: impl Iterator<Item = i64>,
) -> Result<FileMetaData>
where
    T: ArrowPrimitiveType,
{
    let file = File::create(map_master_path(path, master_result_dir()))?;
    let cols = ids.into_iter().zip(hashes).map(move |(i, hash)| {
        (
            i.to_string(),
            Arc::new(PrimitiveArray::<T>::from_iter_values(hash.into_iter())) as ArrayRef,
        )
    });
    write_record_batch(cols, file)
}

pub fn write_record_batch<I, F, W>(iter: I, writer: W) -> Result<FileMetaData>
where
    I: IntoIterator<Item = (F, ArrayRef)>,
    F: AsRef<str>,
    W: Write + Send,
{
    let batch = RecordBatch::try_from_iter(iter)?;
    let props = WriterProperties::default();
    let mut writer = ArrowWriter::try_new(writer, batch.schema(), Some(props))?;
    writer.write(&batch)?;
    Ok(writer.close()?)
}

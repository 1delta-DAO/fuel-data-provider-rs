use chrono::{DateTime, Utc};
use fuels::prelude::Provider;
use fuels::types::BlockHeight;
use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use crate::config::CONFIG;
use crate::domain::service::exception::DataException;
use crate::domain::service::persistence::SyncStatusService;

struct CalcWindow {
    start_block_number: u64,
    start_block_time: DateTime<Utc>,
    end_block_number: u64,
    end_block_time: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct BlockRange {
    pub start_block_number: u64,
    pub start_block_time: DateTime<Utc>,
    pub end_block_number: u64,
    pub end_block_time: DateTime<Utc>,
}

static CALC_WINDOW: Lazy<Mutex<CalcWindow>> = Lazy::new(|| {
    let now = Utc::now();
    Mutex::new(CalcWindow {
        start_block_number: 0,
        start_block_time: now,
        end_block_number: 0,
        end_block_time: now,
        updated_at: now,
    })
});

pub struct BlockchainDataService;

impl BlockchainDataService{

    pub async fn refresh_calc_data(block_range: BlockRange) {
        let mut block_data = CALC_WINDOW.lock().await;
        *block_data = CalcWindow {
            start_block_number: block_range.start_block_number,
            start_block_time: block_range.start_block_time,
            end_block_number: block_range.end_block_number,
            end_block_time: block_range.end_block_time,
            updated_at: Utc::now(),
        };
    }

    /// Fetches block range and updates `CALC_WINDOW`
    pub async fn get_block_range(provider: &Provider) -> BlockRange {
        let sync_status = SyncStatusService::get_status()
            .await.unwrap().ok_or("No sync status service found.").unwrap();

        let mut start_block_number: u64;
        let mut start_block_time: Option<DateTime<Utc>>;

        let end_block_number = provider.latest_block_height().await.unwrap() as u64;
        let end_block_time = provider.latest_block_time().await.unwrap().unwrap();

        if sync_status.block_time != None {
            start_block_number = sync_status.block_number as u64;
            start_block_time = sync_status.block_time;
        }
        else{
            //No data in sync - cold start from config
            start_block_number = CONFIG.default.tx_log_start_block_number.clone();
            start_block_time = None;
            if start_block_number.clone() > 0 {
                start_block_time = provider
                    .block_by_height(BlockHeight::new(start_block_number.clone() as u32))
                    .await.unwrap().ok_or("Start block not found").unwrap().header.time;
            }
        }

        let block_range = BlockRange {
            start_block_number,
            start_block_time: start_block_time.unwrap().to_utc(),
            end_block_number,
            end_block_time,
        };

        // Update CALC_WITH with the new block range
        BlockchainDataService::refresh_calc_data(block_range.clone()).await;

        block_range
    }

    pub async fn get_block_time(provider: &Provider,block_number: &u64) -> Result<DateTime<Utc>, DataException> {
        {
            // Lock the CALC_WINDOW to read the current calculation window data
            let calc_window = CALC_WINDOW.lock().await;

            // Check if the block number is greater than the current end_block_number
            if *block_number > calc_window.end_block_number {
                // Drop the lock before refreshing the calculation window
                drop(calc_window);

                // Refresh the calculation window using the latest data
                log::info!("Refreshing block range - provider query");
                let _ = BlockchainDataService::refresh_calc_data(BlockchainDataService::get_block_range(provider).await);
            }
        }

        // Lock CALC_WINDOW again after refresh to ensure we use updated data
        let calc_window = CALC_WINDOW.lock().await;

        // Validate that the block number is within the updated range
        if *block_number < calc_window.start_block_number || *block_number > calc_window.end_block_number {
            return Err(DataException::BlockTimeEstimatorError(format!(
                "Block number {} is out of the calculated range ({} - {}).",
                block_number, calc_window.start_block_number, calc_window.end_block_number
            )));
        }

        // Calculate the estimated block time using interpolation
        let block_span = calc_window.end_block_number - calc_window.start_block_number;
        let time_span = calc_window.end_block_time - calc_window.start_block_time;

        if block_span == 0 {
            // If there is no difference in block numbers, return the start block time
            return Ok(calc_window.end_block_time);
        }

        // Interpolate to calculate the approximate block time
        let time_per_block = time_span.num_milliseconds() as f64 / block_span as f64;
        let block_offset = *block_number as f64 - calc_window.start_block_number as f64;
        let estimated_time = calc_window.start_block_time
            + chrono::Duration::milliseconds((time_per_block * block_offset) as i64);

        //log::info!("Block: {} - time: {}",block_number,estimated_time);

        Ok(estimated_time)
    }

}
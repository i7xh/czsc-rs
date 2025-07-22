use crate::types::TradeAction;
use chrono::NaiveDateTime;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TradePositionState {
    /// 空仓状态
    Flat,
    /// 持有多头仓位 (手数)
    Long(u32),
    /// 持有空头仓位 (手数)
    Short(u32),
}
impl TradePositionState {
    /// 获取当前持仓状态的手数

    fn gen_trade_actions(volume: u32, action: TradeAction) -> Vec<TradeAction> {
        (0..volume).map(|_| action.clone()).collect()
    }

    pub fn handle_transition(
        &mut self,
        new_volume: i32,
        dt: NaiveDateTime,
        price: f32,
        bar_id: usize,
    ) -> Vec<TradeAction> {
        let mut trade_actions: Vec<TradeAction> = vec![];
        match self {
            TradePositionState::Flat => {
                if new_volume > 0 {
                    let volume = new_volume as u32;
                    trade_actions.extend(Self::gen_trade_actions(
                        volume,
                        TradeAction::OpenLong { dt, price, bar_id },
                    ));
                    *self = TradePositionState::Long(volume);
                } else if new_volume < 0 {
                    let volume = (-new_volume) as u32;
                    trade_actions.extend(Self::gen_trade_actions(
                        volume,
                        TradeAction::OpenShort { dt, price, bar_id },
                    ));
                    *self = TradePositionState::Short(volume);
                }
            }

            TradePositionState::Long(current_volume) => {
                if new_volume > 0 {
                    let new_volume_u32 = new_volume as u32;
                    if new_volume_u32 > *current_volume {
                        // 加仓
                        let trade_volume = new_volume_u32 - *current_volume;
                        trade_actions.extend(Self::gen_trade_actions(
                            trade_volume,
                            TradeAction::OpenLong { dt, price, bar_id },
                        ));
                        *self = TradePositionState::Long(new_volume_u32);
                    } else if new_volume_u32 < *current_volume {
                        // 减仓
                        let trade_volume = *current_volume - new_volume_u32;
                        trade_actions.extend(Self::gen_trade_actions(
                            trade_volume,
                            TradeAction::CloseLong { dt, price, bar_id },
                        ));
                        *self = TradePositionState::Long(new_volume_u32);
                    }
                } else if new_volume < 0 {
                    // 平多开空
                    let short_volume_u32 = (-new_volume) as u32;
                    trade_actions.extend(Self::gen_trade_actions(
                        *current_volume,
                        TradeAction::CloseLong { dt, price, bar_id },
                    ));
                    trade_actions.extend(Self::gen_trade_actions(
                        short_volume_u32,
                        TradeAction::OpenShort { dt, price, bar_id },
                    ));
                    *self = TradePositionState::Short(short_volume_u32);
                } else {
                    // 平仓
                    trade_actions.extend(Self::gen_trade_actions(
                        *current_volume,
                        TradeAction::CloseLong { dt, price, bar_id },
                    ));
                    *self = TradePositionState::Flat;
                }
            }
            TradePositionState::Short(current_volume) => {
                if new_volume < 0 {
                    let new_volume_u32 = (-new_volume) as u32;
                    if new_volume_u32 > *current_volume {
                        // 加仓 (空头)
                        let trade_volume = new_volume_u32 - *current_volume;
                        trade_actions.extend(Self::gen_trade_actions(
                            trade_volume,
                            TradeAction::OpenShort { dt, price, bar_id },
                        ));
                        *self = TradePositionState::Short(new_volume_u32);
                    } else if new_volume_u32 < *current_volume {
                        // 减仓 (空头)
                        let trade_volume = *current_volume - new_volume_u32;
                        trade_actions.extend(Self::gen_trade_actions(
                            trade_volume,
                            TradeAction::CloseShort { dt, price, bar_id },
                        ));
                        *self = TradePositionState::Short(new_volume_u32);
                    }
                } else if new_volume > 0 {
                    let long_volume = new_volume as u32;
                    trade_actions.extend(Self::gen_trade_actions(
                        *current_volume,
                        TradeAction::CloseShort { dt, price, bar_id },
                    ));
                    trade_actions.extend(Self::gen_trade_actions(
                        long_volume,
                        TradeAction::OpenLong { dt, price, bar_id },
                    ));
                    *self = TradePositionState::Long(long_volume);
                } else {
                    trade_actions.extend(Self::gen_trade_actions(
                        *current_volume,
                        TradeAction::CloseShort { dt, price, bar_id },
                    ));
                    *self = TradePositionState::Flat;
                }
            }
        }
        trade_actions
    }
}

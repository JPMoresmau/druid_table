use crate::axis_measure::{LogIdx, TableAxis, VisIdx, VisOffset};
use druid::{EventCtx, Selector};
use std::fmt::Debug;
use std::ops::{Index, IndexMut, Add};
use crate::Remap;

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct CellAddress<T: Copy + Debug> {
    pub row: T,
    pub col: T,
}

impl<T: Copy + Debug> CellAddress<T> {
    pub(crate) fn new(row: T, col: T) -> CellAddress<T> {
        CellAddress { row, col }
    }
}

trait CellAddressMove<O> {
    fn move_by(&self, axis: TableAxis, amount: O) -> Self;
}

impl <O, T: Add<O, Output=T> + Copy + Debug> CellAddressMove<O> for CellAddress<T> {
    fn move_by(&self, axis: TableAxis, amount: O) -> CellAddress<T> {
        let mut moved = (*self).clone();
        moved[axis] = self[axis]  + amount;
        moved
    }
}

impl <T: Copy + Debug> Index<TableAxis> for CellAddress<T>{
    type Output = T;

    fn index(&self, axis: TableAxis) -> &Self::Output {
        match axis {
            TableAxis::Rows => &self.row,
            TableAxis::Columns => &self.col,
        }
    }
}

impl <T: Copy + Debug> IndexMut<TableAxis> for CellAddress<T> {
    fn index_mut(&mut self, axis: TableAxis) -> &mut Self::Output {
        match axis {
            TableAxis::Rows => &mut self.row,
            TableAxis::Columns =>&mut self.col,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SingleCell {
    pub vis: CellAddress<VisIdx>,
    pub log: CellAddress<LogIdx>,
}

impl SingleCell {
    pub fn new(vis: CellAddress<VisIdx>, log: CellAddress<LogIdx>) -> Self {
        SingleCell { vis, log }
    }


}

// Represents a Row or Column. Better name would be nice!
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SingleAxisSlice {
    pub axis: TableAxis,
    pub vis: VisIdx,
    pub log: LogIdx
}

impl SingleAxisSlice {
    pub fn new(axis: TableAxis, vis: VisIdx, log: LogIdx) -> Self {
        SingleAxisSlice { axis, vis, log }
    }
}


#[derive(Debug, Clone)]
pub enum IndicesSelection {
    NoSelection,
    Single(VisIdx, LogIdx),
    //Many(Vec<usize>),
    //Range(from, to)
}

impl IndicesSelection {
    pub(crate) fn vis_index_selected(&self, vis_idx: VisIdx) -> bool {
        match self {
            IndicesSelection::Single(sel_vis, _) => *sel_vis == vis_idx,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TableSelection {
    NoSelection,
    SingleCell(SingleCell),
    SingleSlice(SingleAxisSlice),
    //  SingleRow
    //  Range
    //  Discontiguous
}


pub trait CellDemap{
    fn get_log_idx(&self, axis: TableAxis, vis: &VisIdx) ->Option<LogIdx>;

    fn get_log_cell(&self, vis: &CellAddress<VisIdx>) -> Option<CellAddress<LogIdx>> {
        self.get_log_idx(TableAxis::Rows,&vis.row).map(|row| {
            self.get_log_idx(TableAxis::Columns, &vis.col).map(|col| CellAddress::new(row, col))
        }).flatten()
    }
}

pub trait TableSelectionMod{
    fn new_selection(&self, sel : & TableSelection)->Option<TableSelection>;
}

impl <F: Fn(&TableSelection)->Option<TableSelection>> TableSelectionMod for F{
    fn new_selection(&self, sel: &TableSelection)->Option<TableSelection> {
        self(sel)
    }
}

impl TableSelection {
    pub fn move_focus(&self, axis: TableAxis, amount: VisOffset, cell_demap: &impl CellDemap )->Option<TableSelection>{
        match self{
            Self::NoSelection => {
                let vis_origin = CellAddress::new(VisIdx(0), VisIdx(0));
                cell_demap.get_log_cell(&vis_origin)
                    .map(|log| TableSelection::SingleCell( SingleCell::new(vis_origin, log)))
            },
            Self::SingleCell(SingleCell{vis, ..}) => {
                let new_vis = vis.move_by(axis, amount);
                cell_demap.get_log_cell(&new_vis)
                    .map(|log|TableSelection::SingleCell( SingleCell::new(new_vis, log) ))
            },
            Self::SingleSlice(slice) => {
               // let new_slice = cell_demap
                Some(self.clone())
            }
        }
    }
}


#[derive(Debug, PartialEq)]
pub enum SelectionStatus {
    NotSelected,
    Primary,
    AlsoSelected,
}

impl From<SelectionStatus> for bool {
    fn from(ss: SelectionStatus) -> Self {
        ss != SelectionStatus::NotSelected
    }
}

impl From<SingleCell> for TableSelection {
    fn from(sc: SingleCell) -> Self {
        TableSelection::SingleCell(sc)
    }
}

impl TableSelection {
    pub fn to_axis_selection(&self, axis: TableAxis) -> IndicesSelection {
        match self {
            TableSelection::NoSelection => IndicesSelection::NoSelection,
            TableSelection::SingleCell(sc) => IndicesSelection::Single(sc.vis[axis], sc.log[axis]),
            Self::SingleSlice(single)=>{
                    if single.axis == axis {
                        IndicesSelection::Single(single.vis, single.log)
                    }else{
                        IndicesSelection::NoSelection
                    }
            }
        }
    }

    pub(crate) fn get_cell_status(&self, address: CellAddress<VisIdx>) -> SelectionStatus {
        match self {
            TableSelection::SingleCell(sc) if address == sc.vis => SelectionStatus::Primary,
            _ => SelectionStatus::NotSelected,
        }
    }
}

pub const SELECT_INDICES: Selector<IndicesSelection> =
    Selector::new("druid-builtin.table.select-indices");

pub type SelectionHandler = dyn Fn(&mut EventCtx, &TableSelection);

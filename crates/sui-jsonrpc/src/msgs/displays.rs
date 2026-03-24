// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::fmt::{Display, Formatter};

use tabled::builder::Builder as TableBuilder;
use tabled::settings::style::HorizontalLine;
use tabled::settings::{Panel as TablePanel, Style as TableStyle};

use crate::msgs::{
    SuiArgument, SuiCallArg, SuiCommand, SuiObjectArg, SuiProgrammableMoveCall,
    SuiProgrammableTransactionBlock,
};

pub struct Pretty<'a, T>(pub &'a T);

impl Display for Pretty<'_, SuiProgrammableTransactionBlock> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Pretty(ptb) = self;
        let SuiProgrammableTransactionBlock { inputs, commands } = ptb;
        if !inputs.is_empty() {
            let mut builder = TableBuilder::default();
            for (i, input) in inputs.iter().enumerate() {
                match input {
                    SuiCallArg::Pure(v) => {
                        let pure_arg = if let Some(t) = v.value_type() {
                            format!("{i:<3} Pure Arg: Type: {}, Value: {}", t, v.value())
                        } else {
                            format!("{i:<3} Pure Arg: {}", v.value())
                        };
                        builder.push_record(vec![pure_arg]);
                    }
                    SuiCallArg::Object(SuiObjectArg::ImmOrOwnedObject { object_id, .. }) => {
                        builder.push_record(vec![format!(
                            "{i:<3} Imm/Owned Object ID: {}",
                            object_id
                        )]);
                    }
                    SuiCallArg::Object(SuiObjectArg::SharedObject { object_id, .. }) => {
                        builder.push_record(vec![format!(
                            "{i:<3} Shared Object    ID: {}",
                            object_id
                        )]);
                    }
                    SuiCallArg::Object(SuiObjectArg::Receiving { object_id, .. }) => {
                        builder.push_record(vec![format!(
                            "{i:<3} Receiving Object ID: {}",
                            object_id
                        )]);
                    }
                }
            }

            let mut table = builder.build();
            table.with(TablePanel::header("Input Objects"));
            table.with(
                TableStyle::rounded()
                    .horizontals([(1, HorizontalLine::inherit(TableStyle::modern()))]),
            );
            write!(f, "\n{}", table)?;
        } else {
            write!(f, "\n  No input objects for this transaction")?;
        }

        if !commands.is_empty() {
            let mut builder = TableBuilder::default();
            for (i, c) in commands.iter().enumerate() {
                if i == commands.len() - 1 {
                    builder.push_record(vec![format!("{i:<2} {}", Pretty(c))]);
                } else {
                    builder.push_record(vec![format!("{i:<2} {}\n", Pretty(c))]);
                }
            }
            let mut table = builder.build();
            table.with(TablePanel::header("Commands"));
            table.with(
                TableStyle::rounded()
                    .horizontals([(1, HorizontalLine::inherit(TableStyle::modern()))]),
            );
            write!(f, "\n{}", table)
        } else {
            write!(f, "\n  No commands for this transaction")
        }
    }
}

impl Display for Pretty<'_, SuiCommand> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Pretty(command) = self;
        match command {
            SuiCommand::MakeMoveVec(ty_opt, elems) => {
                write!(f, "MakeMoveVec:\n ┌")?;
                if let Some(ty) = ty_opt {
                    write!(f, "\n │ Type Tag: {ty}")?;
                }
                write!(f, "\n │ Arguments:\n │   ")?;
                write_sep(f, elems.iter().map(Pretty), "\n │   ")?;
                write!(f, "\n └")
            }

            SuiCommand::MoveCall(p) => write!(f, "{}", Pretty(&**p)),

            SuiCommand::MergeCoins(target, coins) => {
                write!(
                    f,
                    "MergeCoins:\n ┌\n │ Target: {}\n │ Coins: \n │   ",
                    Pretty(target)
                )?;
                write_sep(f, coins.iter().map(Pretty), "\n │   ")?;
                write!(f, "\n └")
            }

            SuiCommand::SplitCoins(coin, amounts) => {
                write!(
                    f,
                    "SplitCoins:\n ┌\n │ Coin: {}\n │ Amounts: \n │   ",
                    Pretty(coin)
                )?;
                write_sep(f, amounts.iter().map(Pretty), "\n │   ")?;
                write!(f, "\n └")
            }

            SuiCommand::Publish(deps) => {
                write!(f, "Publish:\n ┌\n │ Dependencies: \n │   ")?;
                write_sep(f, deps, "\n │   ")?;
                write!(f, "\n └")
            }

            SuiCommand::TransferObjects(objs, addr) => {
                write!(f, "TransferObjects:\n ┌\n │ Arguments: \n │   ")?;
                write_sep(f, objs.iter().map(Pretty), "\n │   ")?;
                write!(f, "\n │ Address: {}\n └", Pretty(addr))
            }

            SuiCommand::Upgrade(deps, current_package_id, ticket) => {
                write!(f, "Upgrade:\n ┌\n │ Dependencies: \n │   ")?;
                write_sep(f, deps, "\n │   ")?;
                write!(f, "\n │ Current Package ID: {current_package_id}")?;
                write!(f, "\n │ Ticket: {}", Pretty(ticket))?;
                write!(f, "\n └")
            }
        }
    }
}

impl Display for Pretty<'_, SuiProgrammableMoveCall> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Pretty(move_call) = self;
        let SuiProgrammableMoveCall {
            package,
            module,
            function,
            type_arguments,
            arguments,
        } = move_call;

        write!(
            f,
            "MoveCall:\n ┌\n │ Function:  {} \n │ Module:    {}\n │ Package:   {}",
            function, module, package
        )?;

        if !type_arguments.is_empty() {
            write!(f, "\n │ Type Arguments: \n │   ")?;
            write_sep(f, type_arguments, "\n │   ")?;
        }
        if !arguments.is_empty() {
            write!(f, "\n │ Arguments: \n │   ")?;
            write_sep(f, arguments.iter().map(Pretty), "\n │   ")?;
        }

        write!(f, "\n └")
    }
}

impl Display for Pretty<'_, SuiArgument> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Pretty(argument) = self;

        let output = match argument {
            SuiArgument::GasCoin => "GasCoin".to_string(),
            SuiArgument::Input(i) => format!("Input  {}", i),
            SuiArgument::Result(i) => format!("Result {}", i),
            SuiArgument::NestedResult(j, k) => format!("Nested Result {}: {}", j, k),
        };
        write!(f, "{}", output)
    }
}

/// Originally from `sui_types::transaction::write_sep`.
pub fn write_sep<T: Display>(
    f: &mut Formatter<'_>,
    items: impl IntoIterator<Item = T>,
    sep: &str,
) -> std::fmt::Result {
    let mut xs = items.into_iter();
    let Some(x) = xs.next() else {
        return Ok(());
    };
    write!(f, "{x}")?;
    for x in xs {
        write!(f, "{sep}{x}")?;
    }
    Ok(())
}

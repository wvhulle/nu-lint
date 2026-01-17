//!  Each common command has its own rule for better maintainability.
//!  Less common commands are grouped in the 'other' subrule.

pub mod awk_to_pipeline;
pub mod bat_to_open;
pub mod cat_to_open;
pub mod date_to_date_now;
pub mod df_to_sys_disks;
pub mod external_cd_to_builtin;
pub mod external_ls_to_builtin;
pub mod find_to_glob;
pub mod free_to_sys_mem;
pub mod grep_to_find_or_where;
pub mod head_to_first;
pub mod hostname_to_sys_host;
pub mod pager_to_explore;
pub mod redundant_echo;

pub mod external_sort_to_builtin;
pub mod external_uniq_to_builtin;
pub mod read_to_input;
pub mod sed_to_str_transform;
pub mod tac_to_reverse;
pub mod tail_to_last;
pub mod uname_to_sys_host;
pub mod uptime_to_sys_host;
pub mod users_to_sys_users;
pub mod w_to_sys_users;
pub mod wc_to_length;
pub mod who_to_sys_users;

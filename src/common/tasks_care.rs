use crate::{
    ModelPars,
    agents::tasks::Task,
    full_model::{
        Model,
        person::{Id, Person},
    },
    utilities::{DayInWeek, HourInDay, HourInWeek},
};

pub fn weekly_care_supply(person: &Person, pars: &ModelPars) -> u32 {
    if person.care.need_level > 0 {
        return 0;
    }

    if person.maternity.is_in_maternity() {
        return pars.task_care.care_supply_maternity;
    }

    pars.care.care_supply_by_status[person.work.status.index()]
}

pub fn social_care_demand_per_day(person: &Person, pars: &ModelPars) -> u32 {
    pars.task_care.social_care_demand_per_day[person.care.index()]
}

fn child_care_demand_per_day(person: &Person, pars: &ModelPars) -> u32 {
    let age = person.basic.age.year_month().0;
    if age < pars.task_care.stop_child_care_age {
        if age < pars.task_care.stop_baby_care_age {
            pars.task_care.baby_care_per_day
        } else {
            pars.task_care.child_care_per_day
        }
    } else {
        0
    }
}

pub fn init_care_tasks(p_id: Id, pars: &ModelPars, model: &mut Model) {
    let person = model.pop.alive_mut(p_id);

    assert!(person.task.assigned_tasks.is_empty());
    for t_id in person.task.open_tasks.drain() {
        model.tasks.remove(&t_id).expect("open task did not exist");
    }

    let sc = social_care_demand_per_day(person, pars);
    let cc = child_care_demand_per_day(person, pars);

    // social care demand replaces child care
    let cc = cc.saturating_sub(sc);

    let free_hours = 24u32
        .checked_sub(cc + sc)
        .expect("daily child and social care demand shouldn't exceed 24");

    let first = free_hours / 2;
    let n_sc_1st_half = sc / 2;
    let n_sc_2nd_half = sc - n_sc_1st_half;

    // TODO: focus, urgency

    let mut hour;
    for day in DayInWeek::all_days() {
        hour = first;
        for _h in 0..n_sc_1st_half {
            let hiw = HourInWeek::new(day, HourInDay::new(hour));
            let task = Task::social_care(p_id, hiw);
            let t_id = task.id();
            person.task.open_tasks.insert(&task);
            model.tasks.insert(t_id, task);
            hour += 1;
        }

        for _h in 0..cc {
            let hiw = HourInWeek::new(day, HourInDay::new(hour));
            let task = Task::child_care(p_id, hiw);
            let t_id = task.id();
            person.task.open_tasks.insert(&task);
            model.tasks.insert(t_id, task);
            hour += 1;
        }

        for _h in 0..n_sc_2nd_half {
            let hiw = HourInWeek::new(day, HourInDay::new(hour));
            let task = Task::social_care(p_id, hiw);
            let t_id = task.id();
            person.task.open_tasks.insert(&task);
            model.tasks.insert(t_id, task);
            hour += 1;
        }
    }
}

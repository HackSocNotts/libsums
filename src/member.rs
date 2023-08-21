use chrono::NaiveDate;

pub type StudentId = u32;

#[derive(Debug)]
pub enum MemberType {
    Student,
}

// impl<S> TryFrom<S> for MemberType
// where
//     S: AsRef<str>,
// {
//     fn try_from(value: S) -> Result<Self, Self::Error> {
//         match value {
//             "Student" => Ok(Self::Student),
//             _ => Err(()),
//         }
//     }
// }

#[derive(Debug)]
pub struct Member {
    student_id: StudentId,
    name: String,
    member_type: MemberType,
    subscription_purchased: String,
    date_joined: NaiveDate,
}

impl Member {
    pub fn new(
        student_id: StudentId,
        name: String,
        member_type: MemberType,
        subscription_purchased: String,
        date_joined: NaiveDate,
    ) -> Self {
        Self {
            student_id,
            name,
            member_type,
            subscription_purchased,
            date_joined,
        }
    }
}

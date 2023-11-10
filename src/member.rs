use chrono::NaiveDate;

pub type StudentId = String;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct Member {
    pub student_id: StudentId,
    pub name: String,
    pub member_type: MemberType,
    pub subscription_purchased: String,
    pub date_joined: NaiveDate,
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

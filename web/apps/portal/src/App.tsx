import { Navigate, Route, Routes } from "react-router";
import { Layout } from "./components/Layout";
import { AdminModulesPage } from "./pages/AdminModules";
import { AdminStaffPage } from "./pages/AdminStaff";
import { CoursesPage } from "./pages/Courses";
import { EnrollmentsPage } from "./pages/Enrollments";
import { FacultyPage } from "./pages/Faculty";
import { HomePage } from "./pages/Home";
import { InspectCoursesPage } from "./pages/InspectCourses";
import { InspectStudentsPage } from "./pages/InspectStudents";
import { NotFoundPage } from "./pages/NotFound";
import { ProfilePage } from "./pages/Profile";
import { StudentPage } from "./pages/Student";
import { TeachPage } from "./pages/Teach";
import { TeachOfferingPage } from "./pages/TeachOffering";

/** Paths match the `nav` items that /api/v1/portal/me returns, relative to the /app basename. */
export function App() {
  return (
    <Routes>
      <Route element={<Layout />}>
        <Route index element={<Navigate to="/home" replace />} />
        <Route path="home" element={<HomePage />} />
        <Route path="profile" element={<ProfilePage />} />
        <Route path="courses" element={<CoursesPage />} />
        <Route path="enrollments" element={<EnrollmentsPage />} />
        <Route path="teach" element={<TeachPage />} />
        <Route path="teach/offerings/:id" element={<TeachOfferingPage />} />
        <Route path="faculty" element={<FacultyPage />} />
        <Route path="students/:sub" element={<StudentPage />} />
        <Route path="inspect/students" element={<InspectStudentsPage />} />
        <Route path="inspect/courses" element={<InspectCoursesPage />} />
        <Route path="admin/staff" element={<AdminStaffPage />} />
        <Route path="admin/modules" element={<AdminModulesPage />} />
        <Route path="*" element={<NotFoundPage />} />
      </Route>
    </Routes>
  );
}

# Build stage: package the Spring Boot app with the Maven wrapper from Spring Initializr
FROM eclipse-temurin:25-jdk AS build
WORKDIR /src
COPY .mvn/ .mvn/
COPY mvnw pom.xml ./
RUN chmod +x mvnw && ./mvnw -B -q dependency:go-offline
COPY src/ src/
RUN ./mvnw -B -q -DskipTests package && cp target/*.jar app.jar

# Run stage: only the Java runtime and the jar
FROM eclipse-temurin:25-jre
WORKDIR /app
COPY --from=build /src/app.jar app.jar
ARG GIT_SHA=dev
ENV GIT_SHA=$GIT_SHA
EXPOSE 8080
USER 1000
ENTRYPOINT ["java", "-XX:MaxRAMPercentage=75", "-jar", "app.jar"]
